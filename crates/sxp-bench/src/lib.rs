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

mod budget;
mod classify;
mod error;
mod run;
mod sidecar;
mod synthetic;
mod tune;

pub use budget::{BudgetPlan, assert_same_evaluation, lower_cap, reduce_budget};
pub use classify::{ClassScore, Label, Policy, classify};
pub use error::Error;
pub use run::{Artifacts, Meta, Row, SidecarChoice, benchmark_manifest, report_rows, sizes_hold};
pub use synthetic::{polynomial_signal, recurrence_signal, sha256_hex, trigonometric_signal};
pub use tune::{Point, TuneRow, Tuning, sort_points, tune, tuning_json};
