//! Defines core structs needed for composable and non-composable burst engines


pub mod neuron_model;
pub mod quantization_levels;
pub mod engine;
pub mod connectivity_rule;
pub mod synapse_model;
pub mod visualization;

#[cfg(feature = "std")] // TODO some edits are perfectly fine on no-alloc
pub mod connectome_editing;

