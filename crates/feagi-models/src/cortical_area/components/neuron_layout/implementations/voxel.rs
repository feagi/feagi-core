use crate::cortical_area::components::neuron_layout::neuron_layout_model::{NeuronLayout, NeuronLayoutEnum};
use crate::quantization_levels::burst_engine_index_quantization::BurstEngineIndexQuantization;
use core::marker::PhantomData;
use feagi_data::neurons::wrapped_types::{CorticalNeuronLocalIndex, CorticalVoxelCoordinate, CorticalVoxelDimensions, CorticalVoxelLinearIndex};

/// Defines that the neurons are laid out in xyzd (depth) order linearly in a dense fashion
pub struct NeuronLayoutVoxel<BEIQ: BurstEngineIndexQuantization> {
    pub cortical_dimensions: CorticalVoxelDimensions<BEIQ::NeuronIndexQuant>,
}

impl<BEIQ: BurstEngineIndexQuantization> NeuronLayout<BEIQ> for NeuronLayoutVoxel<BEIQ> {
    const NEURON_LAYOUT_MODEL: NeuronLayoutEnum = NeuronLayoutEnum::Voxel;

    type CorticalLayoutContext = CorticalVoxelDimensions<BEIQ::NeuronIndexQuant>;
    type NeuronLayoutContext = CorticalVoxelCoordinate<BEIQ::NeuronIndexQuant>;

    fn get_cortical_layout_context(&self) -> &Self::CorticalLayoutContext {
        &self.cortical_dimensions
    }

    fn get_neuron_layout_context(&self, neuron_index: &CorticalNeuronLocalIndex<BEIQ::NeuronIndexQuant>) -> Self::NeuronLayoutContext {
        let neuron_index = CorticalVoxelLinearIndex::new(neuron_index.deref());
        self.cortical_dimensions.linear_index_to_coordinate_unchecked(neuron_index)
    }
}
