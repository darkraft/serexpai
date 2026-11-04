//! Baseline exact-size selection. The kernel owns every framed byte.

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

mod codebook;
mod error;
mod select;

pub use codebook::{
    Codebook, assemble_codebook, canonical_bytes, check_family_caps, generate_codebook,
};
pub use error::Error;
pub use select::{
    ArmSearch, Choice, CostCell, LengthPolicy, PartitionKey, SearchCandidate, SearchMode,
    Selection, SpanCost, SpanRecord, SpanTable, all_baselines, allowed_nominals, candidate_id,
    check_length, choose_candidate, encode_baselines, encode_codebook, reduce, search_arm,
    select_baselines, select_codebook, span_cells,
};
