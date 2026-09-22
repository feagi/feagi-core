use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use feagi_basis_collections::generic_data::par_data::{ParData, ParDataStore};

use feagi_basis_collections::spatial_indexing_structs::spatial_index_context::SpatialIndexContext;
use feagi_basis_quantization::prelude::*;
use crate::single_area_collections::voxels::context::{CorticalAreaVoxelCoordinateAxisIndex, CorticalAreaVoxelLinearIndex, CorticalAreaVoxelPotential};

/*
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(
    serialize = "Store: Serialize",
    deserialize = "Store: DeserializeOwned"
))]
pub struct CorticalAreaVoxelData<QI, QP, Store>
where
    QI: QuantizedUnsignedIntegerUnwrappedTrait,
    QP: QuantizedDecimalUnwrappedTrait,
    Store: ParDataStore<Elem = CorticalAreaVoxelPotential<QP>>,
{
    context: SpatialIndexContext<CorticalAreaVoxelCoordinateAxisIndex<QI>, 3>,
    data: ParData<CorticalAreaVoxelLinearIndex<QI>, Store>,
    _p: core::marker::PhantomData<QP>
}


 */






