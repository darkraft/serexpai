//! CLI tokens. One stderr line, no clap paragraph.

use std::io::ErrorKind;

use sxp_kernel::Error as KernelError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CliError {
    token: String,
}

impl CliError {
    pub fn token(token: &'static str) -> Self {
        Self {
            token: token.to_string(),
        }
    }

    pub fn token_owned(token: String) -> Self {
        Self { token }
    }

    pub fn kernel(err: KernelError) -> Self {
        Self {
            token: err.to_string(),
        }
    }

    pub fn io(kind: ErrorKind) -> Self {
        Self {
            token: io_token(kind).to_string(),
        }
    }

    pub fn from_io(err: &std::io::Error) -> Self {
        Self::io(err.kind())
    }

    pub fn as_str(&self) -> &str {
        &self.token
    }
}

pub fn io_token(kind: ErrorKind) -> &'static str {
    match kind {
        ErrorKind::NotFound => "NotFound",
        ErrorKind::AlreadyExists => "AlreadyExists",
        ErrorKind::IsADirectory => "IsADirectory",
        ErrorKind::NotADirectory => "NotADirectory",
        ErrorKind::Unsupported => "Unsupported",
        ErrorKind::CrossesDevices => "CrossDevice",
        _ => "Io",
    }
}

pub fn line(command: Option<&str>, token: &str) -> String {
    match command {
        Some(command) => format!("sxp-cli: {command}: {token}\n"),
        None => format!("sxp-cli: {token}\n"),
    }
}
