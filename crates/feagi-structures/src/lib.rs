//! The core crate for FEAGI. Defines the most common data structures used throughout
#![doc = include_str!("../docs/readme.md")]

#![cfg_attr(not(feature = "std"), no_std)] // Switch to no_std mode if the std feature is disabled

#[cfg(feature = "alloc")]
extern crate alloc;
extern crate core;

mod feagi_json;
mod templates;
mod common_descriptors;
mod feagi_base_error;

pub mod genomic;
pub mod neuron;

pub use feagi_base_error::FeagiBaseError;
pub use feagi_json::FeagiJSON;

// Re-export async macros for convenience
// Note: Macros are exported at crate root via #[macro_export], so we don't need to re-export them here
