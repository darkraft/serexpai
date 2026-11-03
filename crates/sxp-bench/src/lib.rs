//! Benchmark shell. Synthetic signals for the corpus manifest.

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

mod synthetic;

pub use synthetic::{polynomial_signal, recurrence_signal, sha256_hex, trigonometric_signal};
