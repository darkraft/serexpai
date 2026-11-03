use std::fmt;
use std::io::ErrorKind;

/// Typed kernel error. `Display` is the variant name and nothing more.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Error {
    TruncatedHeader,
    BadMagic,
    BadVersion,
    ReservedBits,
    ReservedMode,
    Leb128Overflow,
    NonMinimalLeb128,
    TruncatedLeb128,
    BlockedEmpty,
    BlockedModeUnsupported,
    TruncatedPayload,
    TruncatedTrailer,
    CrcMismatch,
    TrailingBytes,
    LengthOutOfRange,
    ExcessPayload,
    InvalidParameter,
    TruncatedCoder,
    SaturatedRangeCode,
    NonzeroRangeCode,
    ReservedLength,
    ReservedKind,
    UnsupportedKind,
    NonCanonicalLength,
    NonZeroPadding,
    TruncatedBlock,
    BlockLengthMismatch,
    BlockTooLarge,
    UnexpectedMode,
    Io(ErrorKind),
}

impl Error {
    pub const fn name(self) -> &'static str {
        match self {
            Error::TruncatedHeader => "TruncatedHeader",
            Error::BadMagic => "BadMagic",
            Error::BadVersion => "BadVersion",
            Error::ReservedBits => "ReservedBits",
            Error::ReservedMode => "ReservedMode",
            Error::Leb128Overflow => "Leb128Overflow",
            Error::NonMinimalLeb128 => "NonMinimalLeb128",
            Error::TruncatedLeb128 => "TruncatedLeb128",
            Error::BlockedEmpty => "BlockedEmpty",
            Error::BlockedModeUnsupported => "BlockedModeUnsupported",
            Error::TruncatedPayload => "TruncatedPayload",
            Error::TruncatedTrailer => "TruncatedTrailer",
            Error::CrcMismatch => "CrcMismatch",
            Error::TrailingBytes => "TrailingBytes",
            Error::LengthOutOfRange => "LengthOutOfRange",
            Error::ExcessPayload => "ExcessPayload",
            Error::InvalidParameter => "InvalidParameter",
            Error::TruncatedCoder => "TruncatedCoder",
            Error::SaturatedRangeCode => "SaturatedRangeCode",
            Error::NonzeroRangeCode => "NonzeroRangeCode",
            Error::ReservedLength => "ReservedLength",
            Error::ReservedKind => "ReservedKind",
            Error::UnsupportedKind => "UnsupportedKind",
            Error::NonCanonicalLength => "NonCanonicalLength",
            Error::NonZeroPadding => "NonZeroPadding",
            Error::TruncatedBlock => "TruncatedBlock",
            Error::BlockLengthMismatch => "BlockLengthMismatch",
            Error::BlockTooLarge => "BlockTooLarge",
            Error::UnexpectedMode => "UnexpectedMode",
            Error::Io(_) => "Io",
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl std::error::Error for Error {}
