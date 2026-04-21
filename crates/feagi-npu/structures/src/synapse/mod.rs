

pub mod base_traits;
pub mod non_plastic_dimensional;

mod synapse_flags;
mod feagi_npu_synapse_error;
mod dimension_to_dimension_traits;
mod memory_to_dim_traits;

pub use feagi_npu_synapse_error::FeagiNPUSynapseError;
pub use synapse_flags::SynapseFlag;