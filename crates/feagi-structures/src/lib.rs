//! The core crate for FEAGI. Defines the most common data structures used throughout
#![doc = include_str!("../docs/readme.md")]

//#![cfg_attr(not(feature = "std"), no_std)] // Switch to no_std mode if the std feature is disabled


pub use feagi_data; // Expose feagi-data crate
pub use feagi_genome_definitions;
pub use feagi_potential_voxels;


mod feagi_json;
mod feagi_common_error;
pub mod genomic;
pub mod feagi_log;
//pub mod useful_structs_traits_macros;
pub mod neuron;
pub mod wgpu_temp;
pub mod useful_structs;
mod common;

pub use feagi_common_error::FeagiCommonError;