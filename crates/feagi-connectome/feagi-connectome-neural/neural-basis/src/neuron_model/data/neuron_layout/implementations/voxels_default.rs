use crate::neuron_model::data::neuron_layout::{NeuronLayout, VoxelNeuronLayout};
use crate::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_basis::collections::spatial_helpers::axis_order::AxisOrderIncrementing;
use feagi_basis::collections::spatial_helpers::spatial_index_context::SpatialHelper;
use feagi_basis::neurons::{CorticalAreaNeuronCount, CorticalAreaNeuronLocalIndex, DimensionalCorticalAreaDimensions, DimensionalCorticalAreaNeuronCoordinates};
use feagi_basis::prelude::QuantizedUnsignedIntegerTrait;
use serde::{Deserialize, Serialize};

/// Defines that the neurons are laid out in xyzd (depth) order linearly in a dense fashion
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NeuronLayoutVoxelDefault;

/// The data of [`NeuronLayoutVoxelDefault`]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NeuronLayoutVoxelDefaultData<FIQ: FeagiIndexQuantization> {
    pub dimensions: SpatialHelper<FIQ::NeuronIndexQuant, AxisOrderIncrementing<4>, 4>,
}

impl NeuronLayout for NeuronLayoutVoxelDefault {
    type Data<FIQ: FeagiIndexQuantization> = NeuronLayoutVoxelDefaultData<FIQ>;
    type CorticalContext<FIQ: FeagiIndexQuantization> = DimensionalCorticalAreaDimensions<FIQ::NeuronIndexQuant>;
    type PerNeuronContext<FIQ: FeagiIndexQuantization> = DimensionalCorticalAreaNeuronCoordinates<FIQ::NeuronIndexQuant>;

    fn build_new<FIQ: FeagiIndexQuantization>(cortical_context: Self::CorticalContext<FIQ>) -> Self::Data<FIQ> {
        NeuronLayoutVoxelDefaultData {
            dimensions: SpatialHelper::new(cortical_context)
        }
    }

    fn get_neuron_count<FIQ: FeagiIndexQuantization>(data: &Self::Data<FIQ>) -> CorticalAreaNeuronCount<FIQ::NeuronIndexQuant> {
        CorticalAreaNeuronCount::quant_from_usize_unchecked(
            data.dimensions.get_dimensions().spatial_element_count()
        )
    }

    fn get_cortical_layout_context<FIQ: FeagiIndexQuantization>(data: &Self::Data<FIQ>) -> &Self::CorticalContext<FIQ> {
        data.dimensions.get_dimensions()
    }

    fn get_neuron_layout_context<FIQ: FeagiIndexQuantization>(data: &Self::Data<FIQ>, neuron_index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>) -> Self::PerNeuronContext<FIQ> {
        todo!()
        //data.dimensions.linear_to_coordinate(neuron_index.deref())
    }

    fn update_cortical_context<FIQ: FeagiIndexQuantization>(data: &mut Self::Data<FIQ>, cortical_context: Self::CorticalContext<FIQ>) {
        data.dimensions = SpatialHelper::new(cortical_context); // SO we rebuild stride as well
    }
}

impl VoxelNeuronLayout for NeuronLayoutVoxelDefault {
    type AxisOrder = AxisOrderIncrementing<4>;
    const IS_AXIS_ORDER_DEFAULT: bool = true;
    const VOXELS_ALLOWED_DENSE: bool = true;

    fn get_dimensions<FIQ: FeagiIndexQuantization>(data: &Self::Data<FIQ>) -> &DimensionalCorticalAreaDimensions<FIQ::NeuronIndexQuant> {
        Self::get_cortical_layout_context(data)
    }

    fn linear_to_coordinate<FIQ: FeagiIndexQuantization>(data: &Self::Data<FIQ>, neuron_index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>) -> DimensionalCorticalAreaNeuronCoordinates<FIQ::NeuronIndexQuant> {
        Self::get_neuron_layout_context(data, neuron_index)
    }
}
