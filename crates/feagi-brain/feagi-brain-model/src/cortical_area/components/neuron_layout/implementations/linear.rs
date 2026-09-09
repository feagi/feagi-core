use std::marker::PhantomData;
use feagi_data::feagi_data_neuron::neurons::wrapped_types::{CorticalNeuronLocalIndex, NeuronCount};
use feagi_data::feagi_data_neuron::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::models::cortical_area::components::neuron_layout::neuron_layout_model::NeuronLayout;

/// Defines that the neurons are laid out in linear (dense) fashion
pub struct NeuronLayoutLinear<FIQ: FeagiIndexQuantization> {
    pub neuron_count: NeuronCount<FIQ::NeuronIndexQuant>,
}

impl<FIQ: FeagiIndexQuantization> NeuronLayout<FIQ> for NeuronLayoutLinear<FIQ> {
    type CorticalContext = NeuronCount<FIQ::NeuronIndexQuant>;
    type PerNeuronContext = CorticalNeuronLocalIndex<FIQ::NeuronIndexQuant>;

    fn get_neuron_count(&self) -> NeuronCount<FIQ::NeuronIndexQuant> {
        self.neuron_count
    }

    fn get_cortical_layout_context(&self) -> &Self::CorticalContext {
        &self.neuron_count
    }

    fn get_neuron_layout_context(&self, neuron_index: &CorticalNeuronLocalIndex<FIQ::NeuronIndexQuant>) -> Self::PerNeuronContext {
        neuron_index.clone() // no further details lol
    }
}
