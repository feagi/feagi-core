use serde::{Deserialize, Serialize};
use crate::neuron_model::data::neuron_layout::NeuronLayout;
use crate::neuron_model::neuron_model::NeuronLayoutType;
use crate::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_basis::collections::spatial_helpers::axis_order::{AxisOrder, AxisOrderIncrementing};
use feagi_basis::collections::spatial_helpers::spatial_index_context::SpatialHelper;
use feagi_basis::neurons::{CorticalAreaNeuronCount, CorticalAreaNeuronLocalIndex, DimensionalCorticalAreaDimensions, DimensionalCorticalAreaNeuronCoordinates};
use feagi_basis::prelude::QuantizedUnsignedIntegerTrait;

/// Default increasing x y z d indexing order
pub type NeuronLayoutVoxelDefault<FIQ> = NeuronLayoutVoxel<FIQ, AxisOrderIncrementing<4>>;

/// Default increasing x y z d indexing order, for any [`FeagiIndexQuantization`]
pub type NeuronLayoutVoxelDefaultType = NeuronLayoutVoxelType<AxisOrderIncrementing<4>>;

/// Selects [`NeuronLayoutVoxel`] with axis order `AOT`, for any [`FeagiIndexQuantization`]
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct NeuronLayoutVoxelType<AOT: AxisOrder<4> + 'static>(core::marker::PhantomData<AOT>);

impl<AOT: AxisOrder<4> + 'static> NeuronLayoutType for NeuronLayoutVoxelType<AOT> {
    type Layout<FIQ: FeagiIndexQuantization> = NeuronLayoutVoxel<FIQ, AOT>;
}

/// Defines that the neurons are laid out in xyzd (depth) order linearly, with 1 or more neurons
/// per voxel
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(bound = "")]
#[serde(transparent)]
pub struct NeuronLayoutVoxel<FIQ: FeagiIndexQuantization, AOT: AxisOrder<4> + 'static> {
    helper: SpatialHelper<FIQ::NeuronIndexQuant, AOT, 4>
}

impl<FIQ: FeagiIndexQuantization, AOT: AxisOrder<4> + 'static> NeuronLayout<FIQ> for NeuronLayoutVoxel<FIQ, AOT> {
    type CorticalContext = DimensionalCorticalAreaDimensions<FIQ::NeuronIndexQuant>;
    type PerNeuronContext = DimensionalCorticalAreaNeuronCoordinates<FIQ::NeuronIndexQuant>;

    fn new(init_data: Self::CorticalContext) -> Self {
        Self{helper: SpatialHelper::new(init_data) }
    }

    fn get_neuron_count(&self) -> CorticalAreaNeuronCount<FIQ::NeuronIndexQuant> {
        CorticalAreaNeuronCount::quant_from_usize_unchecked(
            self.helper.get_dimensions().spatial_element_count()
        )
    }

    fn get_neuron_layout_context(
        &self,
        neuron_index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
    ) -> Self::PerNeuronContext {
        self.helper.linear_to_coordinate(neuron_index.deref())
    }

    fn update_cortical_context(&mut self, new_cortical_context: Self::CorticalContext) {
        self.helper = SpatialHelper::new(new_cortical_context); // So we rebuild stride as well
    }
}
