//! Defines Data structs that are used to store state tha are used (or extended by) all cortical
//! area implementations

pub(crate) use inverse_outgoing_connection_count::InverseOutgoingConnectionCount;
pub use per_neuron_scratch_flags::PerNeuronScratchFlags;

mod per_neuron_scratch_flags;
mod inverse_outgoing_connection_count;
pub mod cortical_area_property_flags;