mod interneuron_data;
mod neuron_flag;

pub mod no_alloc;
#[cfg(feature = "alloc")]
pub mod alloc;
mod synapse_data;
mod synapse_flag;
mod single_neuron_data;

pub use interneuron_data::InterNeuronData;
pub use neuron_flag::NeuronFlag;
pub use synapse_flag::SynapseFlag;