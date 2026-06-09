//! Error type shared across the FEAGI Trainer engine layers.
//!
//! Errors are structured and explicit (no silent coercion, no fallbacks). Foreign error
//! types are converted to owned messages at the boundary so the public API does not leak
//! third-party types — this keeps the surface stable and Rust/RTOS-migration friendly.

use thiserror::Error;

/// Errors produced by adapters, samplers, metric packs, and orchestration.
#[derive(Debug, Error)]
pub enum TrainerError {
    /// A dataset source could not be parsed into the intermediate representation.
    #[error("dataset parse error: {0}")]
    Parse(String),

    /// A validation gate failed (e.g. schema/compatibility mismatch).
    #[error("validation failed: {0}")]
    Validation(String),

    /// An I/O operation failed.
    #[error("io error: {0}")]
    Io(String),

    /// A configuration value was missing or invalid.
    #[error("invalid configuration: {0}")]
    Config(String),

    /// Evaluation could not be computed for the provided inputs.
    #[error("evaluation error: {0}")]
    Evaluation(String),
}
