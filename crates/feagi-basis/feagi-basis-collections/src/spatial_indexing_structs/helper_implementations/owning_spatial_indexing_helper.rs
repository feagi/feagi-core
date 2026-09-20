use feagi_basis_quantization::prelude::{QuantizedUnsignedIntegerTrait, QuantizedUnsignedIntegerUnwrappedTrait};
use crate::spatial_indexing_structs::axis_order::AxisOrder;
use crate::spatial_indexing_structs::dimensions::SpatialDimensions;
use crate::spatial_indexing_structs::spatial_indexing_helpers::{SpatialIndexingHelper, SpatialIndexingHelperMut};
use crate::spatial_indexing_structs::stride::SpatialStride;

/// An index coordinate converter that owns all the data, for ease of portability
pub struct OwningSpatialIndexingHelper<QLinear, QCoord, QDim, const NUM_DIMS: usize>
where
    QLinear: QuantizedUnsignedIntegerTrait,
    QCoord: QuantizedUnsignedIntegerTrait<QuantType=QLinear::QuantType>,
    QDim: QuantizedUnsignedIntegerTrait<QuantType=QLinear::QuantType>,
{
    dimensions: SpatialDimensions<QDim, NUM_DIMS>,
    stride: SpatialStride<QLinear::QuantType, NUM_DIMS>,
    _p: core::marker::PhantomData<(QLinear, QCoord)>
}

impl<QLinear, QCoord, QDim, const NUM_DIMS: usize>
OwningSpatialIndexingHelper<QLinear, QCoord, QDim, NUM_DIMS>
where
    QLinear: QuantizedUnsignedIntegerTrait,
    QCoord: QuantizedUnsignedIntegerTrait<QuantType=QLinear::QuantType>,
    QDim: QuantizedUnsignedIntegerTrait<QuantType=QLinear::QuantType>,
{

}

impl<QLinear, QCoord, QDim, const NUM_DIMS: usize, const AXIS_ORDER_U8: u8>
SpatialIndexingHelper<QLinear, NUM_DIMS, AXIS_ORDER_U8> for
OwningSpatialIndexingHelper<QLinear, QCoord, QDim, NUM_DIMS>
where
    QLinear: QuantizedUnsignedIntegerTrait,
    QCoord: QuantizedUnsignedIntegerTrait<QuantType=QLinear::QuantType>,
    QDim: QuantizedUnsignedIntegerTrait<QuantType=QLinear::QuantType>,
{
    const AXIS_ORDER: AxisOrder<NUM_DIMS> = AxisOrder::from_u8(AXIS_ORDER_U8);
    
    type CoordinateQuant = QCoord;
    type DimensionsQuant = QDim;

    fn get_dimensions(&self) -> &SpatialDimensions<Self::DimensionsQuant, NUM_DIMS> {
        &self.dimensions
    }

    fn _get_stride(&self) -> &SpatialStride<QLinear::QuantType, NUM_DIMS> {
        &self.stride
    }
}

impl<QLinear, QCoord, QDim, const NUM_DIMS: usize, const AXIS_ORDER_U8: u8>
SpatialIndexingHelperMut<QLinear, NUM_DIMS, AXIS_ORDER_U8> for
OwningSpatialIndexingHelper<QLinear, QCoord, QDim, NUM_DIMS>
where
    QLinear: QuantizedUnsignedIntegerTrait,
    QCoord: QuantizedUnsignedIntegerTrait<QuantType=QLinear::QuantType>,
    QDim: QuantizedUnsignedIntegerTrait<QuantType=QLinear::QuantType>,
{
    fn _get_dimensions_mut(&mut self) -> &mut SpatialDimensions<Self::DimensionsQuant, NUM_DIMS> {
        &mut self.dimensions
    }

    fn _get_stride_mut(&mut self) -> &mut SpatialStride<QLinear::QuantType, NUM_DIMS> {
        &mut self.stride
    }
}