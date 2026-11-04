//! Benchmark tokens. `Display` is the token, with no context string.

use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    RoundTripMismatch,
    SelectorDefect,
    CostOverflow,
    HashMismatch,
    BadManifest,
    EvaluationLeak,
    NewExperiment,
    Kernel(String),
}

impl Error {
    pub fn token(&self) -> &str {
        match self {
            Self::RoundTripMismatch => "RoundTripMismatch",
            Self::SelectorDefect => "SelectorDefect",
            Self::CostOverflow => "CostOverflow",
            Self::HashMismatch => "HashMismatch",
            Self::BadManifest => "BadManifest",
            Self::EvaluationLeak => "EvaluationLeak",
            Self::NewExperiment => "NewExperiment",
            Self::Kernel(token) => token,
        }
    }

    pub fn kernel(err: impl fmt::Display) -> Self {
        Self::Kernel(err.to_string())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.token())
    }
}

impl std::error::Error for Error {}
