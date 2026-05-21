//! The core crate for FEAGI. Defines the most common data structures used throughout
#![doc = include_str!("../docs/readme.md")]

//#![cfg_attr(not(feature = "std"), no_std)] // Switch to no_std mode if the std feature is disabled
// Expose quantization crate
pub use feagi_structures_quantization as quantization;


mod feagi_json;
mod templates;
mod feagi_structures_error;
mod cortical_area__neuron_data_collections;
pub mod genomic;
pub mod feagi_log;
//pub mod useful_structs_traits_macros;
pub mod neuron;
pub mod neuron_voxel;


pub use feagi_structures_error::FeagiStructuresError;