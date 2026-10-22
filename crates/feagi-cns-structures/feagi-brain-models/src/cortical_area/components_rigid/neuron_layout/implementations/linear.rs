use serde::Serialize;
use feagi_basis::neurons::{CorticalAreaNeuronCount, CorticalAreaNeuronLocalIndex};
use crate::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::cortical_area::components_rigid::neuron_layout::NeuronLayout;

/// Defines that the neurons are laid out in linear fashion
#[derive(Clone, Serialize)]
pub struct NeuronLayoutLinear<FIQ: FeagiIndexQuantization> {
    pub neuron_count: CorticalAreaNeuronCount<FIQ::NeuronIndexQuant>,
}

impl<FIQ: FeagiIndexQuantization> NeuronLayout<FIQ> for NeuronLayoutLinear<FIQ> {
    type CorticalContext = CorticalAreaNeuronCount<FIQ::NeuronIndexQuant>;
    type PerNeuronContext = CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>;

    fn get_neuron_count(&self) -> CorticalAreaNeuronCount<FIQ::NeuronIndexQuant> {
        self.neuron_count
    }

    fn get_cortical_layout_context(&self) -> &Self::CorticalContext {
        &self.neuron_count
    }

    fn get_neuron_layout_context(&self, neuron_index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>) -> Self::PerNeuronContext {
        neuron_index.clone() // no further details lol
    }
}
