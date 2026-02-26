//! The core crate for FEAGI. Defines the most common data structures used throughout
#![doc = include_str!("../docs/readme.md")]

pub mod common_macros;
mod feagi_data_error;
mod feagi_json;
mod feagi_signal;
pub mod genomic;
pub mod neuron_voxels;
mod templates;
mod common_descriptors;

pub use feagi_data_error::FeagiDataError;
pub use feagi_json::FeagiJSON;
pub use feagi_signal::{FeagiSignal, FeagiSignalIndex};

// Re-export async macros for convenience
// Note: Macros are exported at crate root via #[macro_export], so we don't need to re-export them here
