//! The core crate for FEAGI. Defines the most common data structures used throughout
#![doc = include_str!("../docs/readme.md")]

mod feagi_json;
mod feagi_signal;
mod templates;
mod common_descriptors;
mod feagi_base_error;

pub mod common_macros;
pub mod genomic;
pub mod neuron;

pub use feagi_base_error::FeagiBaseError;
pub use feagi_json::FeagiJSON;
pub use feagi_signal::{FeagiSignal, FeagiSignalIndex};

// Re-export async macros for convenience
// Note: Macros are exported at crate root via #[macro_export], so we don't need to re-export them here
