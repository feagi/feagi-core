use serde::{Deserialize, Serialize};
use feagi_basis_quantization::prelude::QuantizedUnsignedIntegerTrait;
use crate::spatial_indexing_structs::coordinate::SpatialIndexingCoordinate;
use crate::spatial_indexing_structs::dimensions::{Dimens, SpatialIndexingDimensions};
use crate::spatial_indexing_structs::stride::SpatialIndexingStride;

/// Defines the relationship between spatial indexing and linear indexing
pub trait SpatialIndexingContext<QI: QuantizedUnsignedIntegerTrait, const NUM_DIMS: usize>:
core::fmt::Debug
{
    type LinearIndex: QuantizedUnsignedIntegerTrait<QuantType = QI::QuantType>;
    type Coord<'de>: SpatialIndexingCoordinate<'de, QI, NUM_DIMS>
    where
        Self: 'de;
    type Dims<'de>: SpatialIndexingDimensions<'de, QI, NUM_DIMS, Coord = Self::Coord<'de>>
    where
        Self: 'de;
    //type Stride<'de>: SpatialIndexingStride<'de, QI, NUM_DIMS>
    //where
    //    Self: 'de;

    // TODO things other than stride may be used in the future!

    fn coordinate_to_linear_index(&self, coordinate: &Self::Coord<'_>) -> Self::LinearIndex;

    fn linear_index_to_coordinate(&self, linear_index: Self::LinearIndex) -> Self::Coord<'static>;
}

// TODO makeGeneric
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct DefaultSpatialIndexingContext<QI: QuantizedUnsignedIntegerTrait> {
    pub dimensions: Dimens<QI>
    // stride is constant
}

//impl<QI: QuantizedUnsignedIntegerTrait>





