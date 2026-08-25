use core::marker::PhantomData;
use feagi_data::neurons::neuron_potentials::indexing::NeuronLocalIndex;
use crate::cortical_area::components::neuron_layout::neuron_layout_config::NeuronLayoutConfigTrait;
use crate::cortical_area::components::neuron_layout::neuron_layout_model::{NeuronLayoutModelEnum, NeuronLayoutModelTrait};
use crate::quantization_levels::burst_engine_index_quantization::BurstEngineIndexQuantization;

/// Defines that the neurons are laid out in xyzd (depth) order linearly in a dense fashion 
pub struct NeuronLayoutVoxelModel;

impl NeuronLayoutModelTrait for NeuronLayoutVoxelModel {
    const NEURON_LAYOUT_MODEL: NeuronLayoutModelEnum = NeuronLayoutModelEnum::Dimensional;
}

/// Defines that the neurons are laid out in xyzd (depth) order linearly in a dense fashion 
pub struct NeuronLayoutVoxelConfig<BEIQ: BurstEngineIndexQuantization> {
    pub cortical_dimensions: BEIQ // TODO
}

impl<BEIQ: BurstEngineIndexQuantization> NeuronLayoutConfigTrait<BEIQ> for NeuronLayoutVoxelConfig<BEIQ> {
    type CorticalLayoutContext = ();
    type NeuronLayoutContext = ();

    fn get_cortical_layout_context(&self) -> &Self::CorticalLayoutContext {
        todo!()
    }

    fn get_neuron_layout_context(&self, neuron_index: &NeuronLocalIndex<BEIQ::NeuronIndexQuant>) -> &Self::NeuronLayoutContext {
        todo!()
    }
}
