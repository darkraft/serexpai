use std::fmt;

/// Encoder error. `Display` is the variant name. [`Error::Kernel`] forwards
/// the kernel name, with no extra context.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Kernel(sxp_kernel::Error),
    NoCandidate,
    ThreadPool,
    MissingSpan,
}

impl Error {
    const fn name(&self) -> &'static str {
        match self {
            Self::Kernel(err) => err.name(),
            Self::NoCandidate => "NoCandidate",
            Self::ThreadPool => "ThreadPool",
            Self::MissingSpan => "MissingSpan",
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl std::error::Error for Error {}
