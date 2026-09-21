use feagi_basis_quantization::prelude::QuantizedUnsignedIntegerTrait;
use crate::prelude::{SpatialCoordinate, SpatialDimensions};
use crate::spatial_indexing_structs::axis_order::AxisOrderArray;
use crate::spatial_indexing_structs::SpatialStride;

/// Aids in converting linear indexes to spatial coordinates and back
pub struct SpatialIndexMapper<'a, QDims, const NUM_DIMS: usize>
where
    QDims: QuantizedUnsignedIntegerTrait
{
    dimensions: &'a SpatialDimensions<QDims, NUM_DIMS>,
    axis_order: &'a AxisOrderArray<NUM_DIMS>,
    stride: &'a SpatialStride<NUM_DIMS>
}

impl<'a, QDims, const NUM_DIMS: usize> SpatialIndexMapper<'a, QDims, NUM_DIMS>
where
    QDims: QuantizedUnsignedIntegerTrait
{
    pub fn new(dimensions: &'a SpatialDimensions<QDims, NUM_DIMS>,
               axis_order: &'a AxisOrderArray<NUM_DIMS>,
               stride: &'a SpatialStride<NUM_DIMS>) -> Self {
        Self {
            dimensions, axis_order, stride
        }
    }

    /// Convert from linear index to coordinate.
    pub fn linear_to_coordinate<
        QLinear: QuantizedUnsignedIntegerTrait<QuantType=QDims::QuantType>,
        QCoords: QuantizedUnsignedIntegerTrait<QuantType=QDims::QuantType>,
    >(
        &self,
        linear_index: QLinear,
    ) -> SpatialCoordinate<QCoords, NUM_DIMS> {
        self.stride.linear_to_coordinate(linear_index, self.dimensions)
    }

    /// Convert from coordinate to linear index .
    pub fn coordinate_to_linear<
        QLinear: QuantizedUnsignedIntegerTrait<QuantType=QDims::QuantType>,
        QCoords: QuantizedUnsignedIntegerTrait<QuantType=QDims::QuantType>,
    >(
        &self,
        coordinate: &SpatialCoordinate<QCoords, NUM_DIMS>,
    ) -> QLinear {
        self.stride.coordinate_to_linear(coordinate)
    }
}

