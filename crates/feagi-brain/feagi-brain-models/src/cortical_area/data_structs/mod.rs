
//! Defines Data structs that are used to store state tha are used (or extended by) all cortical
//! area implementations

pub use per_neuron_flags::PerNeuronFlags;
pub(crate) use inverse_outgoing_connection_count::InverseOutgoingConnectionCount;
pub use cortical_area_model_connectome_cortical_data::CorticalAreaModelConnectomeCorticalData;
pub use cortical_area_model_neuron_data::CorticalAreaModelNeuronData;

mod per_neuron_flags;
mod inverse_outgoing_connection_count;
mod cortical_area_model_connectome_cortical_data;
mod cortical_area_model_neuron_data;