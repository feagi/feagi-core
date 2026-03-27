//! The core crate for FEAGI. Defines the most common data structures used throughout
#![doc = include_str!("../docs/readme.md")]

#![cfg_attr(not(feature = "std"), no_std)] // Switch to no_std mode if the std feature is disabled

mod feagi_json;
mod templates;
mod feagi_structures_error;


pub mod genomic;

pub mod neuron_voxels;
pub mod base_quantizable;
pub mod feagi_log;
pub mod descriptors;
pub mod neurons;

pub use feagi_structures_error::FeagiStructuresError;
pub use feagi_json::FeagiJSON;


// Re-export async macros for convenience
// Note: Macros are exported at crate root via #[macro_export], so we don't need to re-export them here
