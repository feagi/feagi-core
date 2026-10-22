use serde::Serialize;
use feagi_basis::collections::spatial_helpers::axis_order::{AxisOrderIncrementing};
use feagi_basis::collections::spatial_helpers::spatial_index_context::SpatialHelper;
use feagi_basis::neurons::{CorticalAreaNeuronCount, CorticalAreaNeuronLocalIndex, DimensionalCorticalAreaDimensions, DimensionalCorticalAreaNeuronAxisPosition, DimensionalCorticalAreaNeuronCoordinates};
use feagi_basis::prelude::QuantizedUnsignedIntegerTrait;
use crate::cortical_area::model::components::neuron_layout::neuron_layout::VoxelNeuronLayout;
use crate::cortical_area::model::components::neuron_layout::NeuronLayout;
use crate::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

/// Defines that the neurons are laid out in xyzd (depth) order linearly in a dense fashion
#[derive(Clone, Serialize)]
pub struct NeuronLayoutVoxelDefault<FIQ: FeagiIndexQuantization> {
    pub dimensions: SpatialHelper<DimensionalCorticalAreaNeuronAxisPosition<FIQ::NeuronIndexQuant>, AxisOrderIncrementing<4>, 4>,
}

impl<FIQ: FeagiIndexQuantization> NeuronLayout<FIQ> for NeuronLayoutVoxelDefault<FIQ> {
    type CorticalContext = DimensionalCorticalAreaDimensions<FIQ::NeuronIndexQuant>;
    type PerNeuronContext = DimensionalCorticalAreaNeuronCoordinates<FIQ::NeuronIndexQuant>;

    fn build_new(cortical_context: Self::CorticalContext) -> Self {
        Self {
            dimensions: SpatialHelper::new(cortical_context)
        }
    }

    fn get_neuron_count(&self) -> CorticalAreaNeuronCount<FIQ::NeuronIndexQuant> {
        CorticalAreaNeuronCount::quant_from_usize_unchecked(
            self.dimensions.get_dimensions().spatial_element_count()
        )
    }

    fn get_cortical_layout_context(&self) -> &Self::CorticalContext {
        self.dimensions.get_dimensions()
    }

    fn get_neuron_layout_context(&self, neuron_index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>) -> Self::PerNeuronContext {
        todo!()
        //self.dimensions.linear_to_coordinate(neuron_index.deref())
    }

    fn update_cortical_context(&mut self, cortical_context: Self::CorticalContext) {
        *&mut self.dimensions = SpatialHelper::new(cortical_context); // SO we rebuild stride as well
    }
}

impl<FIQ: FeagiIndexQuantization> VoxelNeuronLayout<FIQ, AxisOrderIncrementing<4>> for NeuronLayoutVoxelDefault<FIQ> {
    const IS_AXIS_ORDER_DEFAULT: bool = true;
    const VOXELS_ALLOWED_DENSE: bool = true;
}