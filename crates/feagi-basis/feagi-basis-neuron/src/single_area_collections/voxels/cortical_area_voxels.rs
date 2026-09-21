use serde::{Deserialize, Serialize};
use feagi_basis_collections::generic_data::par_data::GenericParData;

use feagi_basis_collections::spatial_indexing_structs::spatial_index_context::SpatialIndexContext;
use feagi_basis_quantization::prelude::*;
use crate::single_area_collections::voxels::context::{CorticalAreaVoxelCoordinateAxisIndex, CorticalAreaVoxelLinearIndex, CorticalAreaVoxelPotential};

#[derive(Debug, Clone, Serialize)]
pub struct CorticalAreaVoxelData<
    QI: QuantizedUnsignedIntegerUnwrappedTrait,
    QP: QuantizedDecimalUnwrappedTrait,
    DataBackEnd: GenericParData< CorticalAreaVoxelLinearIndex<QI>, CorticalAreaVoxelPotential<QP>> + Sized
>
{
    context: SpatialIndexContext<CorticalAreaVoxelCoordinateAxisIndex<QI>, 3>,
    data: DataBackEnd,
    _p: core::marker::PhantomData<QP>
}








