use serde::Serialize;
use feagi_basis::feagi_neuron::wrapped_types::{CorticalAreaNeuronCoordinate, CorticalAreaNeuronDimensions, CorticalAreaNeuronLocalIndex, NeuronCount};
use feagi_basis::prelude::*;
use crate::cortical_area::components::neuron_layout::NeuronLayout;

/// Defines that the neurons are laid out in xyzd (depth) order linearly in a dense fashion
#[derive(Clone, Serialize)]
pub struct NeuronLayoutVoxel<FIQ: FeagiIndexQuantization> {
    pub cortical_dimensions: CorticalAreaNeuronDimensions<FIQ::NeuronIndexQuant>,
}

impl<FIQ: FeagiIndexQuantization> NeuronLayout<FIQ> for NeuronLayoutVoxel<FIQ> {
    type CorticalContext = CorticalAreaNeuronDimensions<FIQ::NeuronIndexQuant>;
    type PerNeuronContext = CorticalAreaNeuronCoordinate<FIQ::NeuronIndexQuant>;

    fn get_neuron_count(&self) -> NeuronCount<FIQ::NeuronIndexQuant> {
        self.cortical_dimensions.number_contained_elements().deref().into()
    }

    fn get_cortical_layout_context(&self) -> &Self::CorticalContext {
        &self.cortical_dimensions
    }

    fn get_neuron_layout_context(&self, neuron_index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>) -> Self::PerNeuronContext {
        let neuron_index = neuron_index.deref().into();
        self.cortical_dimensions.linear_index_to_coordinate_unchecked(neuron_index)
    }
}
