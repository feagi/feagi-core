use serde::{Deserialize, Serialize};
use feagi_basis::collections::spatial_helpers::axis_order::AxisOrderDecrementing;
use feagi_basis::collections::spatial_helpers::spatial_index_context::SpatialHelper;
use feagi_basis::neurons::{CorticalAreaNeuronCount, CorticalAreaNeuronLocalIndex, DimensionalCorticalAreaDimensions, DimensionalCorticalAreaNeuronCoordinates};
use feagi_basis::prelude::QuantizedUnsignedIntegerTrait;
use crate::neuron_model::neuron_layout::{NeuronLayoutData, NeuronLayoutChangePolicy, NeuronLayoutRestrictionError, NeuronLayoutRestrictor, NeuronLayoutSparsity, NeuronLayoutTag};
use crate::quantization_levels::FeagiIndexQuantization;
use crate::visualization::NeuronVisualizationVoxelIndex;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct NeuronLayoutVoxel<FIQ: FeagiIndexQuantization> {
    helper: SpatialHelper<FIQ::NeuronIndexQuant, AxisOrderDecrementing<4>, 4>
}

impl<FIQ: FeagiIndexQuantization> NeuronLayoutVoxel<FIQ> {
    /// Gets the index of the neuron at the given coordinates, the inverse of
    /// `get_neuron_layout_context`. Does NOT check that the coordinates fit this layout!
    pub fn neuron_coordinates_to_index(
        &self,
        coordinates: &DimensionalCorticalAreaNeuronCoordinates<FIQ::NeuronIndexQuant>,
    ) -> CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant> {
        CorticalAreaNeuronLocalIndex::new(self.helper.coordinate_to_linear(coordinates))
    }
}

impl<
    FIQ: FeagiIndexQuantization,
> NeuronLayoutData<FIQ>
for NeuronLayoutVoxel<FIQ> {

    const CHANGE_POLICY: NeuronLayoutChangePolicy = NeuronLayoutChangePolicy::Fixed;
    const SPARSITY: NeuronLayoutSparsity = NeuronLayoutSparsity::Dense;

    type CorticalLevelContext = DimensionalCorticalAreaDimensions<FIQ::NeuronIndexQuant>;
    type NeuronLevelContext = DimensionalCorticalAreaNeuronCoordinates<FIQ::NeuronIndexQuant>;
    type VisualizerContext = NeuronVisualizationVoxelIndex<FIQ::NeuronBitBatchIndexQuant>;

    fn new(init_data: Self::CorticalLevelContext) -> Self {
        Self{helper: SpatialHelper::new(init_data) }
    }

    fn get_number_neurons_to_allocate_for(&self) -> CorticalAreaNeuronCount<FIQ::NeuronIndexQuant> {
        CorticalAreaNeuronCount::quant_from_usize_unchecked(
            self.helper.get_dimensions().spatial_element_count()
        )
    }

    fn get_neuron_layout_context(&self, neuron_index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>) -> Self::NeuronLevelContext {
        self.helper.linear_to_coordinate(neuron_index.deref())
    }
}


/// Neuron Layout Type of Voxel
#[derive(Copy, Clone, Debug)]
pub struct NeuronLayoutTagVoxel;

impl NeuronLayoutTag for NeuronLayoutTagVoxel {
    type Layout<FIQ: FeagiIndexQuantization> = NeuronLayoutVoxel<FIQ>;
}

//region Restrictions

#[derive(Debug, thiserror::Error)]
pub enum NeuronLayoutRestrictionVoxelError {
    #[error("Given Voxel Dimensions exceed stated XYZ Bounds!")]
    ExceedsXYZBounds
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct NeuronLayoutRestrictorVoxelUnbound<FIQ: FeagiIndexQuantization>(core::marker::PhantomData<FIQ>);

impl<FIQ: FeagiIndexQuantization> NeuronLayoutRestrictorVoxelUnbound<FIQ> {
    pub fn new() -> Self { Self(core::marker::PhantomData) }
}

impl<FIQ: FeagiIndexQuantization> NeuronLayoutRestrictor<FIQ>
for NeuronLayoutRestrictorVoxelUnbound<FIQ>
{
    type CorticalLevelContext = DimensionalCorticalAreaDimensions<FIQ::NeuronIndexQuant>;

    fn validate_cortical_context(context: &Self::CorticalLevelContext) -> Result<(), NeuronLayoutRestrictionError> {
        validate_quant_limits::<FIQ>(context)
    }
}

/// Restricts XYZD dimensions to a specific range, lower inclusive higher exclusive
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct NeuronLayoutRestrictorVoxelBound<
    FIQ: FeagiIndexQuantization,
    const X_MIN: usize,
    const Y_MIN: usize,
    const Z_MIN: usize,
    const D_MIN: usize,
    const X_MAX: usize,
    const Y_MAX: usize,
    const Z_MAX: usize,
    const D_MAX: usize,
>(core::marker::PhantomData<FIQ>);

impl<FIQ: FeagiIndexQuantization, const X_MIN: usize, const Y_MIN: usize, const Z_MIN: usize,
    const D_MIN: usize, const X_MAX: usize, const Y_MAX: usize, const Z_MAX: usize, const D_MAX: usize,
> NeuronLayoutRestrictorVoxelBound<FIQ, X_MIN, Y_MIN, Z_MIN, D_MIN, X_MAX, Y_MAX, Z_MAX, D_MAX> {
    pub fn new() -> Self { Self(core::marker::PhantomData) }
}

impl<FIQ: FeagiIndexQuantization,
    const X_MIN: usize,
    const Y_MIN: usize,
    const Z_MIN: usize,
    const D_MIN: usize,
    const X_MAX: usize,
    const Y_MAX: usize,
    const Z_MAX: usize,
    const D_MAX: usize,> NeuronLayoutRestrictor<FIQ> for
NeuronLayoutRestrictorVoxelBound<FIQ, X_MIN, Y_MIN, Z_MIN, D_MIN, X_MAX, Y_MAX, Z_MAX, D_MAX> {
    type CorticalLevelContext = DimensionalCorticalAreaDimensions<FIQ::NeuronIndexQuant>;

    fn validate_cortical_context(context: &Self::CorticalLevelContext) -> Result<(), NeuronLayoutRestrictionError> {
        validate_quant_limits::<FIQ>(context)?;
        let [x, y, z, d] = context.as_slice();
        if  x.quant_to_usize() < X_MIN || x.quant_to_usize() >= X_MAX {
            Err(NeuronLayoutRestrictionVoxelError::ExceedsXYZBounds)?
        };
        if  y.quant_to_usize() < Y_MIN || y.quant_to_usize() >= Y_MAX {
            Err(NeuronLayoutRestrictionVoxelError::ExceedsXYZBounds)?
        };
        if  z.quant_to_usize() < Z_MIN || z.quant_to_usize() >= Z_MAX {
            Err(NeuronLayoutRestrictionVoxelError::ExceedsXYZBounds)?
        };
        if  d.quant_to_usize() < D_MIN || d.quant_to_usize() >= D_MAX {
            Err(NeuronLayoutRestrictionVoxelError::ExceedsXYZBounds)?
        };
        Ok(())
    }
}

fn validate_quant_limits<FIQ: FeagiIndexQuantization>(
    context: &DimensionalCorticalAreaDimensions<FIQ::NeuronIndexQuant>
) -> Result<(), NeuronLayoutRestrictionError> {
    let [x, y, z, d ] = context.as_slice();
    let mut r = x.quant_to_usize().checked_mul(y.quant_to_usize()).ok_or_else(
        || NeuronLayoutRestrictionError::ViolatesQuantizationMax
    )?;
    r = r.checked_mul(z.quant_to_usize()).ok_or_else(
        || NeuronLayoutRestrictionError::ViolatesQuantizationMax
    )?;
    r.checked_mul(d.quant_to_usize()).ok_or_else(
        || NeuronLayoutRestrictionError::ViolatesQuantizationMax
    )?;
    Ok(())
}


//endregion








