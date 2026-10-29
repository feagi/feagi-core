use crate::cortical_area::data::neuron_layout::NeuronLayout;
use crate::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_basis::neurons::{CorticalAreaNeuronCount, CorticalAreaNeuronLocalIndex};
use serde::Serialize;

/// Defines that the neurons are laid out in linear fashion
#[derive(Clone, Serialize)]
pub struct NeuronLayoutLinear<FIQ: FeagiIndexQuantization> {
    pub neuron_count: CorticalAreaNeuronCount<FIQ::NeuronIndexQuant>,
}

impl<FIQ: FeagiIndexQuantization> NeuronLayout<FIQ> for NeuronLayoutLinear<FIQ> {
    type CorticalContext = CorticalAreaNeuronCount<FIQ::NeuronIndexQuant>;
    type PerNeuronContext = CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>;

    fn build_new(cortical_context: Self::CorticalContext) -> Self {
        Self {
            neuron_count: cortical_context
        }
    }

    fn get_neuron_count(&self) -> CorticalAreaNeuronCount<FIQ::NeuronIndexQuant> {
        self.neuron_count
    }

    fn get_cortical_layout_context(&self) -> &Self::CorticalContext {
        &self.neuron_count
    }

    fn get_neuron_layout_context(&self, neuron_index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>) -> Self::PerNeuronContext {
        neuron_index.clone() // no further details lol
    }

    fn update_cortical_context(&mut self, cortical_context: Self::CorticalContext) {
        *&mut self.neuron_count = cortical_context
    }
}
