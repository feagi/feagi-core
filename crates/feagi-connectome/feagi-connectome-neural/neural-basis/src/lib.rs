//! Defines core structs needed for composable and non-composable burst engines


pub mod neuron_model;
pub mod quantization_levels;
pub mod connectivity_rule;
pub mod synapse_model;

// Visualization data is backed by a heap allocated `BitBatchParDataVector`
#[cfg(feature = "alloc")]
pub mod visualization;

#[cfg(feature = "alloc")] // TODO some edits are perfectly fine on no-alloc
pub mod connectome_editing;

// The burst engine trait depends on both connectome editing and visualization
#[cfg(feature = "alloc")]
pub mod engine;

