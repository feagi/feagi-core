use std::marker::PhantomData;
use feagi_data::neurons::neuron::indexing::{NeuronCount, NeuronLocalIndex};
use crate::cortical_area::components::neuron_layout::neuron_layout_config::NeuronLayoutConfigTrait;
use crate::cortical_area::components::neuron_layout::neuron_layout_model::{NeuronLayoutModelEnum, NeuronLayoutModelTrait};
use crate::quantization_levels::burst_engine_index_quantization::BurstEngineIndexQuantization;

/// Defines that the neurons are laid out in linear (dense) fashion
pub struct NeuronLayoutLinearModel;

impl NeuronLayoutModelTrait for NeuronLayoutLinearModel {
    const NEURON_LAYOUT_MODEL: NeuronLayoutModelEnum = NeuronLayoutModelEnum::Linear;
}


/// Defines that the neurons are laid out in linear (dense) fashion
pub struct NeuronLayoutLinearConfig<BEIQ: BurstEngineIndexQuantization> {
    pub neuron_count: NeuronCount<BEIQ::NeuronIndexQuant>,
}

impl<BEIQ: BurstEngineIndexQuantization> NeuronLayoutConfigTrait<BEIQ> for NeuronLayoutLinearConfig<BEIQ> {
    type CorticalLayoutContext = NeuronCount<BEIQ::NeuronIndexQuant>;
    type NeuronLayoutContext = NeuronLocalIndex<BEIQ::NeuronIndexQuant>;

    fn get_cortical_layout_context(&self) -> &Self::CorticalLayoutContext {
        &self.neuron_count
    }

    fn get_neuron_layout_context(&self, neuron_index: &NeuronLocalIndex<BEIQ::NeuronIndexQuant>) -> &Self::NeuronLayoutContext {
        neuron_index // no further details lol
    }
}
