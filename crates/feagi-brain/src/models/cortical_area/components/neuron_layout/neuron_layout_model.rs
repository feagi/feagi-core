use serde::{Deserialize, Serialize};
use feagi_data::feagi_data_neuron::neurons::wrapped_types::{CorticalNeuronLocalIndex, NeuronCount};
use feagi_data::feagi_data_neuron::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

/// Root trait for defining Neuron Layout
pub trait NeuronLayout<FIQ: FeagiIndexQuantization>
{
    /// What data describes the cortical context for a neuron, that has a method that given
    /// the neuron local index, can return the
    type CorticalContext: Clone + Serialize + Deserialize + Sized;

    /// The per neuron context, helps identify a neurons "location" relative to others within a
    /// cortical area
    type PerNeuronContext: Clone + Serialize + Deserialize + Sized;

    /// Gets the (max) number of neurons that this layout encloses
    fn get_neuron_count(&self) -> NeuronCount<FIQ::NeuronIndexQuant>;
    
    fn get_cortical_layout_context(&self) -> &Self::CorticalContext;

    fn get_neuron_layout_context(&self, neuron_index: &CorticalNeuronLocalIndex<FIQ::NeuronIndexQuant>) -> Self::PerNeuronContext;
}
