use feagi_data::neurons::wrapped_types::{CorticalNeuronLocalIndex, NeuronCount};
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

/// Root trait for defining Neuron Layout
pub trait NeuronLayout<FIQ: FeagiIndexQuantization>
{
    /// What layout the neurons are using
    const NEURON_LAYOUT_MODEL: NeuronLayoutEnum;

    /// What data describes the cortical context for a neuron, that has a method that given
    /// the neuron local index, can return the
    type CorticalLayoutContext; // TODO bind with requirements for serialization, clone, etc

    /// The per neuron context, helps identify a neurons "location" relative to others within a
    /// cortical area
    type NeuronLayoutContext; // TODO bind with requirements for serialization, clone, etc

    /// Gets the (max) number of neurons that this layout encloses
    fn get_neuron_count(&self) -> NeuronCount<FIQ::NeuronIndexQuant>;
    
    fn get_cortical_layout_context(&self) -> &Self::CorticalLayoutContext;

    fn get_neuron_layout_context(&self, neuron_index: &CorticalNeuronLocalIndex<FIQ::NeuronIndexQuant>) -> Self::NeuronLayoutContext;
}

pub enum NeuronLayoutEnum {
    Voxel,
    Linear
}