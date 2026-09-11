use serde::{Deserialize, Serialize};
use feagi_basis::feagi_neuron::wrapped_types::{CorticalAreaNeuronLocalIndex, NeuronCount};
use feagi_basis::prelude::*;

/// Root trait for defining Neuron Layout
pub trait NeuronLayout<FIQ: FeagiIndexQuantization>: Clone + Serialize + Sized
{
    /// What data describes the cortical context for a neuron, that has a method that given
    /// the neuron local index, can return the
    type CorticalContext: Clone + Serialize + Sized;

    /// The per neuron context, helps identify a neurons "location" relative to others within a
    /// cortical area
    type PerNeuronContext: Clone + Serialize + Sized;

    /// Gets the (max) number of neurons that this layout encloses
    fn get_neuron_count(&self) -> NeuronCount<FIQ::NeuronIndexQuant>;
    
    fn get_cortical_layout_context(&self) -> &Self::CorticalContext;

    fn get_neuron_layout_context(&self, neuron_index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>) -> Self::PerNeuronContext;
}
