use serde::{Deserialize, Serialize};
use feagi_basis_quantization::prelude::QuantizedUnsignedIntegerTrait;
use crate::prelude::{SpatialCoordinate, SpatialDimensions};
use crate::spatial_indexing_structs::axis_order::AxisOrderArray;
use crate::spatial_indexing_structs::spatial_index_mapper::SpatialIndexMapper;
use crate::spatial_indexing_structs::SpatialStride;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpatialIndexContext<QDims, const NUM_DIMS: usize>
where
    QDims: QuantizedUnsignedIntegerTrait
{
    dimensions: SpatialDimensions<QDims, NUM_DIMS>,
    axis_order: AxisOrderArray<NUM_DIMS>,
    stride: SpatialStride<NUM_DIMS>
}

impl<'a, QDims, const NUM_DIMS: usize> SpatialIndexContext<QDims, NUM_DIMS>
where
    QDims: QuantizedUnsignedIntegerTrait
{
    pub fn new(dimensions: SpatialDimensions<QDims, NUM_DIMS>,
               axis_order: AxisOrderArray<NUM_DIMS>,
               stride: SpatialStride<NUM_DIMS>) -> Self {
        Self {
            dimensions, axis_order, stride
        }
    }

    pub fn get_mapper(&self) -> SpatialIndexMapper<QDims, NUM_DIMS> {
        SpatialIndexMapper::new(
            &self.dimensions,
            &self.axis_order,
            &self.stride
        )
    }
}