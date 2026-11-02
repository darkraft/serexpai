//! Normative kernel. Whole-file raw framing, the polynomial, trigonometric, and recurrence evaluators, and the binary range coder.
//!
//! G0 is bitstream-spec-seed.md SHA-256
//! `f4bea1e5b729db1df957563fd3e8e5bec70c0ab0a382f0f0afc7f9e82196e12e`.
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

mod crc32;
mod error;
mod header;
mod leb128;
mod polynomial;
mod range_coder;
mod raw;
mod recurrence;
mod residual;
mod trigonometric;

pub use crc32::{Crc32, crc32_iso_hdlc};
pub use error::Error;
pub use header::{Header, HeaderBytes, decode_header, encode_whole_file_raw_header};
pub use leb128::{LENGTH_LIMIT, Leb128, decode_leb128, encode_leb128};
pub use polynomial::Polynomial;
pub use range_coder::{
    DecoderState, EncoderState, RangeDecoder, RangeEncoder, decode_schedule, encode_schedule,
    exact_size,
};
pub use raw::{read_whole_file_raw, write_whole_file_raw};
pub use recurrence::Recurrence;
pub use residual::{direct_symbol, reconstruct, residual_symbol, signed_residual};
pub use trigonometric::{Trigonometric, check_sine_lut};
