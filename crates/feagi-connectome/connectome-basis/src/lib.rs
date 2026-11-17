//! Defines core structs needed for composable and non-composable burst engines


pub mod neuron_model;
pub mod quantization_levels;
pub mod connectivity_rule;
pub mod synapse_model;

// Visualization index types are always available (neuron layouts use them). Only the burst
// visualization data, backed by a heap allocated `BitBatchParDataVector`, requires alloc
pub mod visualization;

#[cfg(feature = "alloc")] // TODO some edits are perfectly fine on no-alloc
pub mod connectome_editing;

// The burst engine trait depends on both connectome editing and visualization
#[cfg(feature = "alloc")]
pub mod engine;

