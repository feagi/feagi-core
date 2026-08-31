use std::marker::PhantomData;
use feagi_data::neurons::wrapped_types::{CorticalNeuronLocalIndex, NeuronCount};
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::cortical_area::components::neuron_layout::neuron_layout_model::{NeuronLayout, NeuronLayoutEnum};



/// Defines that the neurons are laid out in linear (dense) fashion
pub struct NeuronLayoutLinear<FIQ: FeagiIndexQuantization> {
    pub neuron_count: NeuronCount<FIQ::NeuronIndexQuant>,
}

impl<FIQ: FeagiIndexQuantization> NeuronLayout<FIQ> for NeuronLayoutLinear<FIQ> {
    
    const NEURON_LAYOUT_MODEL: NeuronLayoutEnum = NeuronLayoutEnum::Linear;
    
    type CorticalLayoutContext = NeuronCount<FIQ::NeuronIndexQuant>;
    type NeuronLayoutContext = CorticalNeuronLocalIndex<FIQ::NeuronIndexQuant>;

    fn get_neuron_count(&self) -> NeuronCount<FIQ::NeuronIndexQuant> {
        self.neuron_count
    }

    fn get_cortical_layout_context(&self) -> &Self::CorticalLayoutContext {
        &self.neuron_count
    }

    fn get_neuron_layout_context(&self, neuron_index: &CorticalNeuronLocalIndex<FIQ::NeuronIndexQuant>) -> Self::NeuronLayoutContext {
        neuron_index.clone() // no further details lol
    }
}
