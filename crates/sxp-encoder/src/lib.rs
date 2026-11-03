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

mod error;
mod select;

pub use error::Error;
pub use select::{
    Choice, LengthPolicy, PartitionKey, SearchMode, Selection, SpanTable, all_baselines,
    allowed_nominals, candidate_id, check_length, choose_candidate, encode_baselines, reduce,
    select_baselines,
};
