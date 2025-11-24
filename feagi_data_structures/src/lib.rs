//! The core crate for FEAGI. Defines the most common data structures used throughout
#![doc = include_str!("../docs/readme.md")]

mod templates;
mod error;
mod feagi_signal;
mod feagi_json;
pub mod shared_enums;
pub mod genomic;
pub mod neuron_voxels;
pub mod common_macros;

pub use error::FeagiDataError as FeagiDataError;
pub use feagi_signal::{FeagiSignal, FeagiSignalIndex};
pub use feagi_json::FeagiJSON;

