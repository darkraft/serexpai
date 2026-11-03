//! `compress` and `decompress`. Help exits 0. Every other parse failure exits 1.

use std::ffi::OsString;
use std::path::PathBuf;

use clap::error::ErrorKind;
use clap::{Arg, Command, value_parser};

use crate::error::CliError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommandName {
    Compress,
    Decompress,
}

impl CommandName {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Compress => "compress",
            Self::Decompress => "decompress",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Parsed {
    pub command: CommandName,
    pub src: PathBuf,
    pub dst: PathBuf,
}

pub enum ParseResult {
    Run(Parsed),
    Help(String),
    Usage(CliError, Option<CommandName>),
}

pub fn parse<I, S>(args: I) -> ParseResult
where
    I: IntoIterator<Item = S>,
    S: Into<OsString>,
{
    let owned: Vec<OsString> = args.into_iter().map(Into::into).collect();
    let hinted = owned.get(1).and_then(|arg| arg.to_str()).and_then(name_of);
    match command().try_get_matches_from(owned) {
        Ok(matches) => match matches.subcommand() {
            Some(("compress", sub)) => take(CommandName::Compress, sub),
            Some(("decompress", sub)) => take(CommandName::Decompress, sub),
            _ => ParseResult::Usage(CliError::token("Usage"), hinted),
        },
        Err(err)
            if err.kind() == ErrorKind::DisplayHelp || err.kind() == ErrorKind::DisplayVersion =>
        {
            ParseResult::Help(err.to_string())
        }
        Err(_) => ParseResult::Usage(CliError::token("Usage"), hinted),
    }
}

fn take(command: CommandName, sub: &clap::ArgMatches) -> ParseResult {
    match (sub.get_one::<PathBuf>("src"), sub.get_one::<PathBuf>("dst")) {
        (Some(src), Some(dst)) => ParseResult::Run(Parsed {
            command,
            src: src.clone(),
            dst: dst.clone(),
        }),
        _ => ParseResult::Usage(CliError::token("Usage"), Some(command)),
    }
}

fn name_of(arg: &str) -> Option<CommandName> {
    match arg {
        "compress" => Some(CommandName::Compress),
        "decompress" => Some(CommandName::Decompress),
        _ => None,
    }
}

fn command() -> Command {
    Command::new("sxp-cli")
        .disable_help_subcommand(true)
        .disable_version_flag(true)
        .subcommand_required(true)
        .subcommand(paths(Command::new("compress")))
        .subcommand(paths(Command::new("decompress")))
}

fn paths(cmd: Command) -> Command {
    cmd.arg(
        Arg::new("src")
            .required(true)
            .value_parser(value_parser!(PathBuf)),
    )
    .arg(
        Arg::new("dst")
            .required(true)
            .value_parser(value_parser!(PathBuf)),
    )
}
