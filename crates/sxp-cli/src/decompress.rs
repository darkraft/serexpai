//! Decompress is CLI to kernel. This module does not call the encoder.

use std::io::Write;
use std::path::Path;

use sxp_kernel::read_file;

use crate::error::CliError;
use crate::fs::Fs;
use crate::install::{abandon, create_temp, prepare};

pub fn decompress(fs: &mut dyn Fs, src: &Path, dst: &Path) -> Result<(), CliError> {
    let _source = prepare(fs, src, dst)?;
    let mut reader = fs.open_read(src).map_err(|err| CliError::from_io(&err))?;
    let (temp, mut file) = create_temp(fs, dst)?;
    let outcome = (|| {
        match read_file(&mut reader, &mut file) {
            Ok(()) => {}
            Err(err) => return Err(CliError::kernel(err)),
        }
        file.flush().map_err(|err| CliError::from_io(&err))?;
        file.sync_all().map_err(|err| CliError::from_io(&err))?;
        Ok(())
    })();
    drop(file);
    drop(reader);
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
