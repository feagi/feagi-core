use feagi_data::neurons::neuron_potentials::indexing::NeuronLocalIndex;
use crate::quantization_levels::burst_engine_index_quantization::BurstEngineIndexQuantization;


/// Root trait for defining Neuron Layout
pub trait NeuronLayoutConfigTrait<BEIQ: BurstEngineIndexQuantization>
{
    /// What data describes the cortical context for a neuron, that has a method that given
    /// the neuron local index, can return the 
    type CorticalLayoutContext; // TODO bind with requirements for serialization, clone, etc
    
    /// The per neuron context, helps identify a neurons "location" relative to others within a
    /// cortical area
    type NeuronLayoutContext; // TODO bind with requirements for serialization, clone, etc
    
    fn get_cortical_layout_context(&self) -> &Self::CorticalLayoutContext;
    
    fn get_neuron_layout_context(&self, neuron_index: &NeuronLocalIndex<BEIQ::NeuronIndexQuant>) -> &Self::NeuronLayoutContext;
}