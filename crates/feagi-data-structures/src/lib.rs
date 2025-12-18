//! The core crate for FEAGI. Defines the most common data structures used throughout
#![doc = include_str!("../docs/readme.md")]

pub mod common_macros;
mod error;
mod feagi_json;
mod feagi_signal;
pub mod genomic;
pub mod neuron_voxels;
pub mod shared_enums;
mod templates;

pub use error::FeagiDataError;
pub use feagi_json::FeagiJSON;
pub use feagi_signal::{FeagiSignal, FeagiSignalIndex};
