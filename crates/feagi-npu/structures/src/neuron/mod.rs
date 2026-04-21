mod feagi_npu_neuron_error;

pub mod dimensional_neurons;
pub mod memory;
pub mod base_traits;
pub mod base_dimension_traits;

pub mod flags;
pub(crate) mod defaults;
// pub mod shared_functions; // TODO

pub use feagi_npu_neuron_error::FeagiNPUNeuronError;

// TODO cortical area duplication