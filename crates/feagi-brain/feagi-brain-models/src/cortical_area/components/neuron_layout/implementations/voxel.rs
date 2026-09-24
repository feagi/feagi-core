use serde::Serialize;
use feagi_basis::feagi_neuron::single_area_collections::neurons::{CorticalAreaNeuronCount, CorticalAreaNeuronLocalIndex, DimensionalCorticalAreaDimensions, DimensionalCorticalAreaNeuronCoordinates};
use feagi_basis::prelude::*;
use crate::cortical_area::components::neuron_layout::NeuronLayout;

/// Defines that the neurons are laid out in xyzd (depth) order linearly in a dense fashion
#[derive(Clone, Serialize)]
pub struct NeuronLayoutVoxel<FIQ: FeagiIndexQuantization> {
    pub cortical_dimensions: DimensionalCorticalAreaDimensions<FIQ::NeuronIndexQuant>,
}

impl<FIQ: FeagiIndexQuantization> NeuronLayout<FIQ> for NeuronLayoutVoxel<FIQ> {
    type CorticalContext = DimensionalCorticalAreaDimensions<FIQ::NeuronIndexQuant>;
    type PerNeuronContext = DimensionalCorticalAreaNeuronCoordinates<FIQ::NeuronIndexQuant>;

    fn get_neuron_count(&self) -> CorticalAreaNeuronCount<FIQ::NeuronIndexQuant> {
        CorticalAreaNeuronCount::quant_from_usize_unchecked(
            self.cortical_dimensions.spatial_element_count()
        )
    }

    fn get_cortical_layout_context(&self) -> &Self::CorticalContext {
        &self.cortical_dimensions
    }

    fn get_neuron_layout_context(
        &self, 
        neuron_index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>
    ) -> Self::PerNeuronContext {
        todo!()
        
        //let neuron_index = neuron_index.deref().into();
        //self.cortical_dimensions.linear_index_to_coordinate_unchecked(neuron_index)
    }
}
