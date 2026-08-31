use crate::cortical_area::components::neuron_layout::neuron_layout_model::{NeuronLayout, NeuronLayoutEnum};
use feagi_data::neurons::wrapped_types::{CorticalNeuronCoordinate, CorticalNeuronDimensions, CorticalNeuronLocalIndex, NeuronCount};
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

/// Defines that the neurons are laid out in xyzd (depth) order linearly in a dense fashion
pub struct NeuronLayoutVoxel<FIQ: FeagiIndexQuantization> {
    pub cortical_dimensions: CorticalNeuronDimensions<FIQ::NeuronIndexQuant>,
}

impl<FIQ: FeagiIndexQuantization> NeuronLayout<FIQ> for NeuronLayoutVoxel<FIQ> {
    const NEURON_LAYOUT_MODEL: NeuronLayoutEnum = NeuronLayoutEnum::Voxel;

    type CorticalLayoutContext = CorticalNeuronDimensions<FIQ::NeuronIndexQuant>;
    type NeuronLayoutContext = CorticalNeuronCoordinate<FIQ::NeuronIndexQuant>;

    fn get_neuron_count(&self) -> NeuronCount<FIQ::NeuronIndexQuant> {
        self.cortical_dimensions.number_contained_elements().deref().into()
    }

    fn get_cortical_layout_context(&self) -> &Self::CorticalLayoutContext {
        &self.cortical_dimensions
    }

    fn get_neuron_layout_context(&self, neuron_index: &CorticalNeuronLocalIndex<FIQ::NeuronIndexQuant>) -> Self::NeuronLayoutContext {
        let neuron_index = neuron_index.deref().into();
        self.cortical_dimensions.linear_index_to_coordinate_unchecked(neuron_index)
    }
}
