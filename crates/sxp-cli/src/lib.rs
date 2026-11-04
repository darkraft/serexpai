//! File commands. `compress` and `decompress` install output by rename.
//!
//! The compress recipe is `fixture-v1` until the production recipe is chosen.
//! Decompress calls [`sxp_kernel::read_file`] and does not load a codebook.

#![forbid(unsafe_code)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::todo,
    clippy::unimplemented,
    clippy::unreachable
)]

use std::ffi::OsString;
use std::io::{self, Write};

mod args;
mod benchmark;
mod compress;
mod decompress;
mod error;
mod fs;
mod install;

pub use args::CommandName;
pub use error::{CliError, line};
pub use fs::{Fail, Fs, MemFs, NodeKind, StdFs, normalize, parent_of};

pub struct Output {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

pub fn run() -> i32 {
    let mut fs = StdFs;
    let output = run_with(std::env::args_os(), &mut fs);
    let mut stdout = io::stdout();
    let mut stderr = io::stderr();
    let _ = stdout.write_all(output.stdout.as_bytes());
    let _ = stderr.write_all(output.stderr.as_bytes());
    let _ = stdout.flush();
    let _ = stderr.flush();
    output.code
}

pub fn run_with<I, S>(args: I, fs: &mut dyn Fs) -> Output
where
    I: IntoIterator<Item = S>,
    S: Into<OsString>,
{
    match args::parse(args) {
        args::ParseResult::Help(text) => Output {
            code: 0,
            stdout: text,
            stderr: String::new(),
        },
        args::ParseResult::Usage(err, command) => fail(command.map(CommandName::as_str), &err),
        args::ParseResult::Run(parsed) => match parsed {
            args::Parsed::Compress { src, dst } => match compress::compress(fs, &src, &dst) {
                Ok(()) => quiet(),
                Err(err) => fail(Some("compress"), &err),
            },
            args::Parsed::Decompress { src, dst } => match decompress::decompress(fs, &src, &dst) {
                Ok(()) => quiet(),
                Err(err) => fail(Some("decompress"), &err),
            },
            args::Parsed::Benchmark {
                out,
                manifest,
                gate,
            } => match benchmark::benchmark(fs, &out, &manifest, &gate) {
                Ok(report) => Output {
                    code: 0,
                    stdout: report,
                    stderr: String::new(),
                },
                Err(err) => fail(Some("benchmark"), &err),
            },
        },
    }
}

fn quiet() -> Output {
    Output {
        code: 0,
        stdout: String::new(),
        stderr: String::new(),
    }
}

fn fail(command: Option<&str>, err: &CliError) -> Output {
    Output {
        code: 1,
        stdout: String::new(),
        stderr: line(command, err.as_str()),
    }
}
