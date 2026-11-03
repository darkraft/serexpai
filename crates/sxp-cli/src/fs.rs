//! Filesystem seam. `StdFs` is the process. `MemFs` injects failures.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::io::{self, ErrorKind, Read, Write};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use std::os::unix::fs::MetadataExt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeKind {
    File,
    Dir,
    Symlink,
    Other,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Meta {
    pub len: u64,
    pub dev: u64,
    pub ino: u64,
    pub kind: NodeKind,
}

pub trait InstallWrite: Write {
    fn sync_all(&mut self) -> io::Result<()>;
}

pub trait Fs {
    fn symlink_meta(&self, path: &Path) -> io::Result<Meta>;
    fn meta(&self, path: &Path) -> io::Result<Meta>;
    fn open_read(&mut self, path: &Path) -> io::Result<Box<dyn Read>>;
    fn create_new(&mut self, path: &Path) -> io::Result<Box<dyn InstallWrite>>;
    fn rename(&mut self, from: &Path, to: &Path) -> io::Result<()>;
    fn remove(&mut self, path: &Path) -> io::Result<()>;
}

pub struct StdFs;

impl Fs for StdFs {
    fn symlink_meta(&self, path: &Path) -> io::Result<Meta> {
        meta_of(&std::fs::symlink_metadata(path)?, false)
    }

    fn meta(&self, path: &Path) -> io::Result<Meta> {
        meta_of(&std::fs::metadata(path)?, true)
    }

    fn open_read(&mut self, path: &Path) -> io::Result<Box<dyn Read>> {
        Ok(Box::new(File::open(path)?))
    }

    fn create_new(&mut self, path: &Path) -> io::Result<Box<dyn InstallWrite>> {
        let file = OpenOptions::new().write(true).create_new(true).open(path)?;
        Ok(Box::new(StdFile(file)))
    }

    fn rename(&mut self, from: &Path, to: &Path) -> io::Result<()> {
        std::fs::rename(from, to)
    }

    fn remove(&mut self, path: &Path) -> io::Result<()> {
        std::fs::remove_file(path)
    }
}

struct StdFile(File);

impl Write for StdFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

impl InstallWrite for StdFile {
    fn sync_all(&mut self) -> io::Result<()> {
        self.0.sync_all()
    }
}

fn meta_of(meta: &std::fs::Metadata, followed: bool) -> io::Result<Meta> {
    let kind = if meta.is_symlink() && !followed {
        NodeKind::Symlink
    } else if meta.is_dir() {
        NodeKind::Dir
    } else if meta.is_file() {
        NodeKind::File
    } else {
        NodeKind::Other
    };
    Ok(Meta {
        len: meta.len(),
        dev: meta.dev(),
        ino: meta.ino(),
        kind,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fail {
    None,
    /// Fail the 1-based `open_read` call.
    OpenRead(u32),
    /// Fail the first `read` of a file opened after this is set.
    ReadBody,
    Write,
    Flush,
    Sync,
    Rename,
    /// `symlink_meta` of a path with a file name returns `PermissionDenied`.
    Stat,
}

struct Node {
    kind: NodeKind,
    bytes: Vec<u8>,
    reported_len: u64,
    ino: u64,
    target: PathBuf,
}

struct Inner {
    nodes: BTreeMap<PathBuf, Node>,
    next_ino: u64,
    fail: Fail,
    opens: u32,
    creates: u32,
}

pub struct MemFs {
    inner: Rc<RefCell<Inner>>,
}

impl MemFs {
    pub fn new() -> Self {
        let mut nodes = BTreeMap::new();
        nodes.insert(
            PathBuf::from("."),
            Node {
                kind: NodeKind::Dir,
                bytes: Vec::new(),
                reported_len: 0,
                ino: 1,
                target: PathBuf::new(),
            },
        );
        Self {
            inner: Rc::new(RefCell::new(Inner {
                nodes,
                next_ino: 2,
                fail: Fail::None,
                opens: 0,
                creates: 0,
            })),
        }
    }

    pub fn fail(&mut self, fail: Fail) {
        self.inner.borrow_mut().fail = fail;
    }

    pub fn opens(&self) -> u32 {
        self.inner.borrow().opens
    }

    pub fn creates(&self) -> u32 {
        self.inner.borrow().creates
    }

    pub fn insert_file(&mut self, path: &Path, bytes: Vec<u8>, reported_len: u64) {
        self.insert(path, NodeKind::File, bytes, reported_len, None);
    }

    pub fn insert_dir(&mut self, path: &Path) {
        self.insert(path, NodeKind::Dir, Vec::new(), 0, None);
    }

    pub fn insert_symlink(&mut self, path: &Path, target: &Path) {
        self.insert(path, NodeKind::Symlink, Vec::new(), 0, Some(target));
    }

    pub fn insert_other(&mut self, path: &Path) {
        self.insert(path, NodeKind::Other, Vec::new(), 0, None);
    }

    pub fn bytes(&self, path: &Path) -> Option<Vec<u8>> {
        let key = normalize(path);
        self.inner
            .borrow()
            .nodes
            .get(&key)
            .map(|node| node.bytes.clone())
    }

    pub fn contains(&self, path: &Path) -> bool {
        self.inner.borrow().nodes.contains_key(&normalize(path))
    }

    pub fn temp_names(&self) -> Vec<PathBuf> {
        self.inner
            .borrow()
            .nodes
            .keys()
            .filter(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.starts_with(".sxp-tmp-") && name != ".sxp-tmp-keep")
            })
            .cloned()
            .collect()
    }

    fn insert(
        &mut self,
        path: &Path,
        kind: NodeKind,
        bytes: Vec<u8>,
        reported_len: u64,
        target: Option<&Path>,
    ) {
        let mut inner = self.inner.borrow_mut();
        let ino = inner.next_ino;
        inner.next_ino = inner.next_ino.saturating_add(1);
        inner.nodes.insert(
            normalize(path),
            Node {
                kind,
                reported_len,
                bytes,
                ino,
                target: target.map(normalize).unwrap_or_default(),
            },
        );
    }
}

impl Default for MemFs {
    fn default() -> Self {
        Self::new()
    }
}

impl Fs for MemFs {
    fn symlink_meta(&self, path: &Path) -> io::Result<Meta> {
        let inner = self.inner.borrow();
        if matches!(inner.fail, Fail::Stat) && path.file_name().is_some() {
            return Err(io::Error::from(ErrorKind::PermissionDenied));
        }
        let node = inner
            .nodes
            .get(&normalize(path))
            .ok_or_else(|| io::Error::from(ErrorKind::NotFound))?;
        Ok(Meta {
            len: node.reported_len,
            dev: 1,
            ino: node.ino,
            kind: node.kind,
        })
    }

    fn meta(&self, path: &Path) -> io::Result<Meta> {
        let inner = self.inner.borrow();
        let node = follow(&inner, path)?;
        Ok(Meta {
            len: node.reported_len,
            dev: 1,
            ino: node.ino,
            kind: node.kind,
        })
    }

    fn open_read(&mut self, path: &Path) -> io::Result<Box<dyn Read>> {
        let mut inner = self.inner.borrow_mut();
        inner.opens = inner.opens.saturating_add(1);
        if let Fail::OpenRead(n) = inner.fail
            && inner.opens == n
        {
            return Err(io::Error::from(ErrorKind::PermissionDenied));
        }
        let node = follow(&inner, path)?;
        if node.kind == NodeKind::Dir {
            return Err(io::Error::from(ErrorKind::IsADirectory));
        }
        if node.kind == NodeKind::Other {
            return Err(io::Error::from(ErrorKind::Unsupported));
        }
        let fail_body = matches!(inner.fail, Fail::ReadBody);
        Ok(Box::new(MemReader {
            data: node.bytes.clone(),
            pos: 0,
            fail_body,
        }))
    }

    fn create_new(&mut self, path: &Path) -> io::Result<Box<dyn InstallWrite>> {
        let key = normalize(path);
        let mut inner = self.inner.borrow_mut();
        if inner.nodes.contains_key(&key) {
            return Err(io::Error::from(ErrorKind::AlreadyExists));
        }
        let parent = parent_of(&key);
        let parent_kind = follow(&inner, &parent)?.kind;
        if parent_kind != NodeKind::Dir {
            return Err(io::Error::from(ErrorKind::NotADirectory));
        }
        inner.creates = inner.creates.saturating_add(1);
        let ino = inner.next_ino;
        inner.next_ino = inner.next_ino.saturating_add(1);
        inner.nodes.insert(
            key.clone(),
            Node {
                kind: NodeKind::File,
                bytes: Vec::new(),
                reported_len: 0,
                ino,
                target: PathBuf::new(),
            },
        );
        Ok(Box::new(MemWriter {
            inner: Rc::clone(&self.inner),
            path: key,
        }))
    }

    fn rename(&mut self, from: &Path, to: &Path) -> io::Result<()> {
        let mut inner = self.inner.borrow_mut();
        if matches!(inner.fail, Fail::Rename) {
            return Err(io::Error::from(ErrorKind::CrossesDevices));
        }
        let from_key = normalize(from);
        let to_key = normalize(to);
        let node = inner
            .nodes
            .remove(&from_key)
            .ok_or_else(|| io::Error::from(ErrorKind::NotFound))?;
        let dest_is_dir = inner
            .nodes
            .get(&to_key)
            .is_some_and(|existing| existing.kind == NodeKind::Dir);
        if dest_is_dir {
            inner.nodes.insert(from_key, node);
            return Err(io::Error::from(ErrorKind::IsADirectory));
        }
        inner.nodes.insert(to_key, node);
        Ok(())
    }

    fn remove(&mut self, path: &Path) -> io::Result<()> {
        let mut inner = self.inner.borrow_mut();
        if inner.nodes.remove(&normalize(path)).is_none() {
            return Err(io::Error::from(ErrorKind::NotFound));
        }
        Ok(())
    }
}

fn follow<'a>(inner: &'a Inner, path: &Path) -> io::Result<&'a Node> {
    let mut key = normalize(path);
    let mut seen = Vec::new();
    for _ in 0..8 {
        if seen.iter().any(|old: &PathBuf| old == &key) {
            return Err(symlink_loop());
        }
        seen.push(key.clone());
        let node = inner
            .nodes
            .get(&key)
            .ok_or_else(|| io::Error::from(ErrorKind::NotFound))?;
        if node.kind != NodeKind::Symlink {
            return Ok(node);
        }
        let next = if node.target.is_absolute() {
            node.target.clone()
        } else {
            parent_of(&key).join(&node.target)
        };
        key = normalize(&next);
    }
    Err(symlink_loop())
}

/// Linux `ELOOP`. `ErrorKind::FilesystemLoop` is still unstable on this toolchain.
const ELOOP: i32 = 40;

pub(crate) fn symlink_loop() -> io::Error {
    io::Error::other("symlink loop")
}

pub(crate) fn is_symlink_loop(err: &io::Error) -> bool {
    err.raw_os_error() == Some(ELOOP)
        || err
            .get_ref()
            .is_some_and(|inner| inner.to_string() == "symlink loop")
}

struct MemReader {
    data: Vec<u8>,
    pos: usize,
    fail_body: bool,
}

impl Read for MemReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.fail_body {
            self.fail_body = false;
            return Err(io::Error::from(ErrorKind::PermissionDenied));
        }
        let rest = self.data.get(self.pos..).unwrap_or(&[]);
        if rest.is_empty() || buf.is_empty() {
            return Ok(0);
        }
        let n = rest.len().min(buf.len());
        let src = rest.get(..n).unwrap_or(&[]);
        let dst = buf.get_mut(..n).unwrap_or(&mut []);
        dst.copy_from_slice(src);
        self.pos = self.pos.saturating_add(n);
        Ok(n)
    }
}

struct MemWriter {
    inner: Rc<RefCell<Inner>>,
    path: PathBuf,
}

impl Write for MemWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut inner = self.inner.borrow_mut();
        if matches!(inner.fail, Fail::Write) {
            return Err(io::Error::from(ErrorKind::WriteZero));
        }
        let node = inner
            .nodes
            .get_mut(&self.path)
            .ok_or_else(|| io::Error::from(ErrorKind::NotFound))?;
        node.bytes.extend_from_slice(buf);
        node.reported_len = u64::try_from(node.bytes.len()).unwrap_or(u64::MAX);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        if matches!(self.inner.borrow().fail, Fail::Flush) {
            return Err(io::Error::from(ErrorKind::WriteZero));
        }
        Ok(())
    }
}

impl InstallWrite for MemWriter {
    fn sync_all(&mut self) -> io::Result<()> {
        if matches!(self.inner.borrow().fail, Fail::Sync) {
            return Err(io::Error::from(ErrorKind::WriteZero));
        }
        Ok(())
    }
}

pub fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => match out.file_name() {
                Some(name) if name == ".." => out.push(".."),
                Some(_) => {
                    out.pop();
                }
                None => {
                    if !out.has_root() {
                        out.push("..");
                    }
                }
            },
            other => out.push(other),
        }
    }
    if out.as_os_str().is_empty() {
        out.push(".");
    }
    out
}

pub fn parent_of(path: &Path) -> PathBuf {
    match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.to_path_buf(),
        _ => PathBuf::from("."),
    }
}
