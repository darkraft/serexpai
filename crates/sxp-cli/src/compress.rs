//! Compress calls the fixture codebook. The recipe name is `fixture-v1` and
//! the label is `fixture`, not `production` and not `benchmark`. D-09 stays open.

use std::io::{self, ErrorKind, Read, Write};
use std::path::Path;

use sxp_encoder::{LengthPolicy, SearchMode, all_baselines, encode_codebook, generate_codebook};
use sxp_kernel::{self, LENGTH_LIMIT, read_file};

use crate::error::CliError;
use crate::fs::Fs;
use crate::install::{abandon, create_temp, prepare};

pub fn compress(fs: &mut dyn Fs, src: &Path, dst: &Path) -> Result<(), CliError> {
    let source = prepare(fs, src, dst)?;
    if source.len >= LENGTH_LIMIT {
        return Err(CliError::kernel(sxp_kernel::Error::LengthOutOfRange));
    }
    let mut reader = fs.open_read(src).map_err(|err| CliError::from_io(&err))?;
    let body = read_exact(&mut reader, source.len)?;
    drop(reader);
    let codebook =
        generate_codebook("fixture-v1").map_err(|err| CliError::token_owned(err.to_string()))?;
    let encoded = encode_codebook(
        &body,
        &all_baselines(),
        &codebook,
        LengthPolicy::All,
        SearchMode::HeaderPrune,
        0,
    )
    .map_err(|err| CliError::token_owned(err.to_string()))?;

    let (temp, mut file) = create_temp(fs, dst)?;
    let outcome = (|| {
        write_all(&mut file, &encoded)?;
        let mut check = fs.open_read(&temp).map_err(|err| CliError::from_io(&err))?;
        let mut compared = Compare::new(&body);
        match read_file(&mut check, &mut compared) {
            Ok(()) => {}
            Err(sxp_kernel::Error::Io(ErrorKind::InvalidData)) => {
                return Err(CliError::token("ValidationMismatch"));
            }
            Err(err) => return Err(CliError::kernel(err)),
        }
        if !compared.complete() {
            return Err(CliError::token("ValidationMismatch"));
        }
        file.flush().map_err(|err| CliError::from_io(&err))?;
        file.sync_all().map_err(|err| CliError::from_io(&err))?;
        Ok(())
    })();
    drop(file);
    if let Err(err) = outcome {
        abandon(fs, &temp);
        return Err(err);
    }
    if let Err(err) = fs.rename(&temp, dst) {
        abandon(fs, &temp);
        return Err(CliError::from_io(&err));
    }
    Ok(())
}

fn read_exact(reader: &mut dyn Read, len: u64) -> Result<Vec<u8>, CliError> {
    let mut out = Vec::new();
    let mut left = len.saturating_add(1);
    let mut buf = [0u8; 8192];
    while left > 0 {
        let want = usize::try_from(left.min(8192)).unwrap_or(8192);
        let slot = buf.get_mut(..want).ok_or_else(|| CliError::token("Io"))?;
        match reader.read(slot) {
            Ok(0) => break,
            Ok(n) => {
                let piece = slot.get(..n).ok_or_else(|| CliError::token("Io"))?;
                out.extend_from_slice(piece);
                left = left.saturating_sub(u64::try_from(n).unwrap_or(0));
            }
            Err(err) if err.kind() == ErrorKind::Interrupted => continue,
            Err(err) => return Err(CliError::from_io(&err)),
        }
    }
    let got = u64::try_from(out.len()).unwrap_or(u64::MAX);
    if got != len {
        return Err(CliError::token("LengthChanged"));
    }
    Ok(out)
}

fn write_all(file: &mut dyn Write, mut bytes: &[u8]) -> Result<(), CliError> {
    while !bytes.is_empty() {
        match file.write(bytes) {
            Ok(0) => return Err(CliError::token("Io")),
            Ok(n) => {
                bytes = bytes.get(n..).ok_or_else(|| CliError::token("Io"))?;
            }
            Err(err) if err.kind() == ErrorKind::Interrupted => continue,
            Err(err) => return Err(CliError::from_io(&err)),
        }
    }
    Ok(())
}

struct Compare<'a> {
    expected: &'a [u8],
    pos: usize,
}

impl<'a> Compare<'a> {
    fn new(expected: &'a [u8]) -> Self {
        Self { expected, pos: 0 }
    }

    fn complete(&self) -> bool {
        self.pos == self.expected.len()
    }
}

impl Write for Compare<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let available = self.expected.len().saturating_sub(self.pos);
        if buf.len() > available {
            return Err(io::Error::new(ErrorKind::InvalidData, "mismatch"));
        }
        let end = self
            .pos
            .checked_add(buf.len())
            .ok_or_else(|| io::Error::new(ErrorKind::InvalidData, "mismatch"))?;
        let slice = self
            .expected
            .get(self.pos..end)
            .ok_or_else(|| io::Error::new(ErrorKind::InvalidData, "mismatch"))?;
        if slice != buf {
            return Err(io::Error::new(ErrorKind::InvalidData, "mismatch"));
        }
        self.pos = end;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
