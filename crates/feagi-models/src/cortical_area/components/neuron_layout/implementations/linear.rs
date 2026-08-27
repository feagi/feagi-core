use std::marker::PhantomData;
use feagi_data::neurons::wrapped_types::{CorticalNeuronLocalIndex, NeuronCount};
use crate::cortical_area::components::neuron_layout::neuron_layout_model::{NeuronLayout, NeuronLayoutEnum};
use crate::quantization_levels::burst_engine_index_quantization::BurstEngineIndexQuantization;



/// Defines that the neurons are laid out in linear (dense) fashion
pub struct NeuronLayoutLinear<BEIQ: BurstEngineIndexQuantization> {
    pub neuron_count: NeuronCount<BEIQ::NeuronIndexQuant>,
}

impl<BEIQ: BurstEngineIndexQuantization> NeuronLayout<BEIQ> for NeuronLayoutLinear<BEIQ> {
    
    const NEURON_LAYOUT_MODEL: NeuronLayoutEnum = NeuronLayoutEnum::Linear;
    
    type CorticalLayoutContext = NeuronCount<BEIQ::NeuronIndexQuant>;
    type NeuronLayoutContext = CorticalNeuronLocalIndex<BEIQ::NeuronIndexQuant>;

    fn get_cortical_layout_context(&self) -> &Self::CorticalLayoutContext {
        &self.neuron_count
    }

    fn get_neuron_layout_context(&self, neuron_index: &CorticalNeuronLocalIndex<BEIQ::NeuronIndexQuant>) -> Self::NeuronLayoutContext {
        neuron_index.clone() // no further details lol
    }
}
