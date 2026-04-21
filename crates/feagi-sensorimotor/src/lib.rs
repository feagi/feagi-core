// Compat shim bridging the pre-refactor `feagi_structures` surface
// (FeagiDataError, FeagiSignal[Index], NeuronDepth, legacy `define_*` macros)
// to the current quant-generic `feagi-structures`. See `_compat.rs` for the
// migration rationale.
#[doc(hidden)]
pub mod _compat;

pub mod caching;
mod neuron_voxel_coding;

pub mod configuration;
mod connector_cache;
pub mod data_pipeline;
pub mod data_types;
mod feagi_interfaces;
pub mod feedbacks;
pub mod single_voxel_decode;
pub mod wrapped_io_data;

pub use connector_cache::ConnectorCache;
