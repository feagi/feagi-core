//! This library contains Quantization Traits, Types, and Macros to generate those types easily
//! It also contains bit packing systems

// mod bit_packing; // TODO
// mod common; // TODO

mod flags; // only contains macros, no need to export

// TODO move to own subcrate?
mod feagi_quantized_hardware_error;
mod quantization_levels;

pub mod quantizable_base;
pub mod quantizable_spatial;
pub mod percentages;
pub mod quantizable_collections;

pub use paste;
pub use feagi_quantized_hardware_error::FeagiQuantizedHardwareError;
pub use quantization_levels::{CorticalAreaNeuronQuantization, NPUGlobalQuantization, QuantizationLevel};