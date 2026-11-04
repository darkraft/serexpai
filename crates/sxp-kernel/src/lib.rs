//! Normative kernel. Whole-file raw framing, baseline Block framing, the polynomial, trigonometric, and recurrence evaluators, the binary range coder, and the order-0 probability model.
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

mod block;
mod crc32;
mod error;
mod file;
mod formula_wire;
mod header;
mod leb128;
mod model;
mod polynomial;
mod range_coder;
mod raw;
mod recurrence;
mod residual;
mod trigonometric;

pub use block::{
    AccountedFile, BLOCK_WINDOW, BlockKind, BlockPlan, BlockTrace, BlockView, ByteAccount,
    EncodedBlock, MAX_ENCODED_BLOCK, NominalLength, actual_len, decode_block, decode_block_model,
    encode_block, encode_block_model, is_canonical, read_blocked_file, trace_block,
    write_blocked_accounted, write_blocked_file, write_raw_accounted,
};
pub use crc32::{Crc32, crc32_iso_hdlc};
pub use error::Error;
pub use file::read_file;
pub use formula_wire::{formula_section, section_len};
pub use header::{Header, HeaderBytes, decode_header, encode_whole_file_raw_header};
pub use leb128::{LENGTH_LIMIT, Leb128, decode_leb128, encode_leb128};
pub use model::{
    DecisionTrace, InitialProbability, ModelConfig, NodeState, Order0Model, STRAWMAN_CAP,
    STRAWMAN_P, STRAWMAN_RECIP,
};
pub use polynomial::Polynomial;
pub use range_coder::{
    DecoderState, EncoderState, RangeDecoder, RangeEncoder, decode_schedule, encode_schedule,
    exact_size,
};
pub use raw::{read_whole_file_raw, write_whole_file_raw};
pub use recurrence::Recurrence;
pub use residual::{direct_symbol, reconstruct, residual_symbol, signed_residual};
pub use trigonometric::{Trigonometric, check_sine_lut};
