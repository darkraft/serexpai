//! Destination install.
//!
//! Success replaces the destination directory entry by renaming a temp file in
//! the same parent. There is no prompt and no `--force`. Failure does not
//! remove the destination. An existing destination stays byte-identical. A
//! missing destination stays missing.
//!
//! The temp name is `.sxp-tmp-<dest-file-name>.<pid>.<n>`, created with
//! `create_new`. On `AlreadyExists`, `n` increases at most 100 times. On
//! failure, only that temp path is removed. A sibling, including another
//! `.sxp-tmp-*` file, stays. `EXDEV` and every other rename error are failures.
//! The command does not copy onto the destination.
//!
//! These are refused before a temp file is created:
//! - the destination has no file name, a trailing slash, a trailing `/.`, or is
//!   `.` or `..` (`Usage`)
//! - the parent is missing (`NotFound`). An empty parent means `.`
//! - the parent is not a directory (`NotADirectory`)
//! - the destination exists and is a directory (`IsADirectory`), or a socket
//!   or device (`Unsupported`). A symlink is allowed; `rename` replaces the
//!   symlink itself
//! - the source is missing (`NotFound`). Followed metadata says the source is
//!   a directory, including a symlink to a directory (`IsADirectory`)
//! - the paths are the same directory entry, by absolute lexical spelling or by
//!   `(st_dev, st_ino)` from followed metadata (`SameFile`). Hard links and a
//!   symlink to the same inode are refusals. Lexical comparison does not follow
//!   symlinks
//!
//! The source is opened before the temp file. The destination path is never
//! opened for write or truncate. Compress uses the followed metadata length
//! for the `2^35` check. Decompress ignores the encoded file's byte length.
//!
//! The new file uses the process create mode. It does not copy the old
//! destination's mode. The parent directory is not fsynced after `rename`.
//! A failure after a successful `rename` cannot be rolled back.

use std::ffi::OsString;
use std::io::ErrorKind;
use std::path::{Component, Path, PathBuf};

use crate::error::CliError;
use crate::fs::{self, Fs, InstallWrite, NodeKind};

pub struct Source {
    pub len: u64,
}

pub fn prepare(fs: &dyn Fs, src: &Path, dst: &Path) -> Result<Source, CliError> {
    destination_name(dst)?;
    parent_is_dir(fs, &fs::parent_of(dst))?;
    match fs.symlink_meta(dst) {
        Ok(meta) => match meta.kind {
            NodeKind::Dir => return Err(CliError::token("IsADirectory")),
            NodeKind::Other => return Err(CliError::token("Unsupported")),
            NodeKind::File | NodeKind::Symlink => {}
        },
        Err(err) if err.kind() == ErrorKind::NotFound => {}
        Err(err) => return Err(CliError::from_io(&err)),
    }
    let src_meta = match fs.meta(src) {
        Ok(meta) => meta,
        Err(err) => return Err(CliError::from_io(&err)),
    };
    if src_meta.kind == NodeKind::Dir {
        return Err(CliError::token("IsADirectory"));
    }
    if src_meta.kind != NodeKind::File {
        return Err(CliError::token("Unsupported"));
    }
    if lexical_same(src, dst) {
        return Err(CliError::token("SameFile"));
    }
    match fs.meta(dst) {
        Ok(dst_meta) if dst_meta.dev == src_meta.dev && dst_meta.ino == src_meta.ino => {
            return Err(CliError::token("SameFile"));
        }
        Ok(_) => {}
        Err(err) if err.kind() == ErrorKind::NotFound || fs::is_symlink_loop(&err) => {}
        Err(err) => return Err(CliError::from_io(&err)),
    }
    Ok(Source { len: src_meta.len })
}

pub fn create_temp(
    fs: &mut dyn Fs,
    dst: &Path,
) -> Result<(PathBuf, Box<dyn InstallWrite>), CliError> {
    let name = destination_name(dst)?;
    let parent = fs::parent_of(dst);
    let pid = std::process::id();
    for n in 0..100u32 {
        let mut file_name = OsString::from(".sxp-tmp-");
        file_name.push(&name);
        file_name.push(".");
        file_name.push(pid.to_string());
        file_name.push(".");
        file_name.push(n.to_string());
        let path = parent.join(file_name);
        match fs.create_new(&path) {
            Ok(file) => return Ok((path, file)),
            Err(err) if err.kind() == ErrorKind::AlreadyExists => continue,
            Err(err) => return Err(CliError::from_io(&err)),
        }
    }
    Err(CliError::token("AlreadyExists"))
}

pub fn abandon(fs: &mut dyn Fs, temp: &Path) {
    let _ = fs.remove(temp);
}

fn parent_is_dir(fs: &dyn Fs, parent: &Path) -> Result<(), CliError> {
    match fs.symlink_meta(parent) {
        Ok(meta) if meta.kind == NodeKind::Dir => Ok(()),
        Ok(meta) if meta.kind == NodeKind::Symlink => match fs.meta(parent) {
            Ok(followed) if followed.kind == NodeKind::Dir => Ok(()),
            Ok(_) => Err(CliError::token("NotADirectory")),
            Err(err) => Err(CliError::from_io(&err)),
        },
        Ok(_) => Err(CliError::token("NotADirectory")),
        Err(err) => Err(CliError::from_io(&err)),
    }
}

fn destination_name(path: &Path) -> Result<OsString, CliError> {
    let raw = path.as_os_str().as_encoded_bytes();
    if raw.ends_with(b"/") || raw.ends_with(b"/.") {
        return Err(CliError::token("Usage"));
    }
    if matches!(
        path.components().next_back(),
        Some(Component::CurDir | Component::ParentDir)
    ) {
        return Err(CliError::token("Usage"));
    }
    match path.file_name() {
        Some(name) if name != "." && name != ".." => Ok(name.to_os_string()),
        _ => Err(CliError::token("Usage")),
    }
}

fn lexical_same(src: &Path, dst: &Path) -> bool {
    match (std::path::absolute(src), std::path::absolute(dst)) {
        (Ok(left), Ok(right)) => left == right,
        _ => fs::normalize(src) == fs::normalize(dst),
    }
}
