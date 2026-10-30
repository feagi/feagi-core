use crate::neuron_model::data::neuron_layout::NeuronLayout;
use crate::neuron_model::neuron_model::NeuronLayoutType;
use crate::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_basis::neurons::{CorticalAreaNeuronCount, CorticalAreaNeuronLocalIndex};
use serde::{Deserialize, Serialize};
use feagi_basis::prelude::QuantizedUnsignedIntegerTrait;

/// Selects [`NeuronLayoutLinear`], for any [`FeagiIndexQuantization`]
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct NeuronLayoutLinearType;

impl NeuronLayoutType for NeuronLayoutLinearType {
    type Layout<FIQ: FeagiIndexQuantization> = NeuronLayoutLinear<FIQ>;
}

/// Defines that the neurons are laid out in linear fashion
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct NeuronLayoutLinear<FIQ: FeagiIndexQuantization> {
    neuron_count: CorticalAreaNeuronCount<FIQ::NeuronIndexQuant>
}

impl<FIQ: FeagiIndexQuantization> NeuronLayout<FIQ> for NeuronLayoutLinear<FIQ> {
    type CorticalContext = CorticalAreaNeuronCount<FIQ::NeuronIndexQuant>;
    type PerNeuronContext = CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>;

    // yes these are trivial lol

    fn new(init_data: Self::CorticalContext) -> Self {
        Self {neuron_count: init_data}
    }

    fn get_neuron_count(&self) -> CorticalAreaNeuronCount<FIQ::NeuronIndexQuant> {
        self.neuron_count
    }

    fn get_neuron_layout_context(
        &self,
        neuron_index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
    ) -> Self::PerNeuronContext {
        debug_assert!(neuron_index.quant_to_usize() < self.neuron_count.quant_to_usize());
        neuron_index.clone() // lol
    }

    fn update_cortical_context(&mut self, new_cortical_context: Self::CorticalContext) {
        self.neuron_count = new_cortical_context;
    }
}
