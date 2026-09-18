use serde::Serialize;
use feagi_basis_quantization::prelude::QuantizedUnsignedIntegerTrait;
use super::spatial_indexing_structs::coordinate::SpatialCoordinate;
use super::spatial_indexing_structs::dimensions::SpatialDimensions;
use super::spatial_indexing_structs::stride::SpatialStride;

// NOTE: Keeping this a trait as we may have other context types in the future
/// Defines the relationship between spatial indexing and linear indexing. 
pub trait SpatialIndexingContext<QI: QuantizedUnsignedIntegerTrait, const NUM_DIMS: usize>:
    core::fmt::Debug
{
    type LinearIndex: QuantizedUnsignedIntegerTrait<QuantType = QI::QuantType>;
    fn coordinate_to_linear_index(&self, coordinate: &SpatialCoordinate<QI, NUM_DIMS>) -> Self::LinearIndex;

    fn linear_index_to_coordinate(&self, linear_index: Self::LinearIndex) -> SpatialCoordinate<QI, NUM_DIMS>;
}

/// Generic default context carrying owned dimensions and stride metadata.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct DefaultSpatialIndexingContext<QI: QuantizedUnsignedIntegerTrait, const NUM_DIMS: usize> {
    pub dimensions: SpatialDimensions<QI, NUM_DIMS>,
    pub stride: SpatialStride<QI, NUM_DIMS>,
}

impl<QI: QuantizedUnsignedIntegerTrait, const NUM_DIMS: usize> DefaultSpatialIndexingContext<QI, NUM_DIMS> {
    /// Construct from owned dimensions and stride values.
    pub fn new(
        dimensions: SpatialDimensions<QI, NUM_DIMS>,
        stride: SpatialStride<QI, NUM_DIMS>,
    ) -> Self {
        Self { dimensions, stride }
    }
}

/// Convenience type alias for the default generic coordinate type.
pub type DefaultSpatialCoordinate<QI, const NUM_DIMS: usize> = SpatialCoordinate<QI, NUM_DIMS>;
