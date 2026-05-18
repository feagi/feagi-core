//! The core crate for FEAGI. Defines the most common data structures used throughout
#![doc = include_str!("../docs/readme.md")]

//#![cfg_attr(not(feature = "std"), no_std)] // Switch to no_std mode if the std feature is disabled



pub mod base_feagi_types;
pub mod genomic;
pub mod feagi_log;
pub mod useful_structs_traits_macros;

pub use feagi_structures_error::FeagiStructuresError;

mod feagi_json;
mod templates;
mod feagi_structures_error;
mod quantization_level;
mod cortical_area__neuron_data_collections;
mod neuron;
mod feagi_models;
mod neuron_dynamics;


