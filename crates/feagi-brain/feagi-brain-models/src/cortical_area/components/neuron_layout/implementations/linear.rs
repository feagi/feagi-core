
use serde::Serialize;
use feagi_basis::feagi_neuron::wrapped_types::{CorticalAreaNeuronLocalIndex, NeuronCount};
use feagi_basis::prelude::*;
use crate::cortical_area::components::neuron_layout::NeuronLayout;

/// Defines that the neurons are laid out in linear fashion
#[derive(Clone, Serialize)]
pub struct NeuronLayoutLinear<FIQ: FeagiIndexQuantization> {
    pub neuron_count: NeuronCount<FIQ::NeuronIndexQuant>,
}

impl<FIQ: FeagiIndexQuantization> NeuronLayout<FIQ> for NeuronLayoutLinear<FIQ> {
    type CorticalContext = NeuronCount<FIQ::NeuronIndexQuant>;
    type PerNeuronContext = CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>;

    fn get_neuron_count(&self) -> NeuronCount<FIQ::NeuronIndexQuant> {
        self.neuron_count
    }

    fn get_cortical_layout_context(&self) -> &Self::CorticalContext {
        &self.neuron_count
    }

    fn get_neuron_layout_context(&self, neuron_index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>) -> Self::PerNeuronContext {
        neuron_index.clone() // no further details lol
    }
}
