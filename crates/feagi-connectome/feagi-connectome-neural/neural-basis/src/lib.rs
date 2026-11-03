//! Defines core structs needed for composable and non-composable burst engines


pub mod neuron_model;
pub mod quantization_levels;
pub mod engine;
pub mod connectivity_rule;
pub mod synapse_model;

#[cfg(feature = "std")]
pub mod connectome_editing;
pub mod visualization;

