use crate::spatial_indexing_structs::coordinate::SpatialCoordinate;
use crate::spatial_indexing_structs::dimensions::SpatialDimensions;
use crate::spatial_indexing_structs::stride::SpatialStride;
use feagi_basis_quantization::prelude::{QuantizedUnsignedIntegerTrait, QuantizedUnsignedIntegerUnwrappedTrait};
use crate::spatial_indexing_structs::axis_order::AxisOrder;

pub trait SpatialIndexingHelper<
    QI: QuantizedUnsignedIntegerUnwrappedTrait<QuantType=QI>,
    const NUM_DIMS: usize,
    const AXIS_ORDER_U8: u8
> {
    const AXIS_ORDER: AxisOrder<NUM_DIMS> = AxisOrder::from_u8(AXIS_ORDER_U8);

    type LinearIndex: QuantizedUnsignedIntegerTrait<QuantType=QI>;

    type CoordinateQuant: QuantizedUnsignedIntegerTrait<QuantType=QI>;

    type DimensionsQuant: QuantizedUnsignedIntegerTrait<QuantType=QI>;

    /// Get the current in use dimensions
    fn get_dimensions(&self) -> &SpatialDimensions<Self::DimensionsQuant, NUM_DIMS>;

    /// Update dimensions, including updating the stride as well
    fn update_dimensions(&mut self, new_dims: SpatialDimensions<Self::DimensionsQuant, NUM_DIMS>) {
        self._get_stride_mut().update_stride(&new_dims, &Self::AXIS_ORDER.generate_axis_order_array());
        *self._get_dimensions_mut() = new_dims;
    }

    // NOTE: Yes, this is a bit ugly here, but the best way above to handle this

    #[doc(hidden)]
    fn _get_stride(&self) -> &SpatialStride<QI, NUM_DIMS>;

    #[doc(hidden)]
    fn _get_dimensions_mut(&mut self) -> &mut SpatialDimensions<Self::DimensionsQuant, NUM_DIMS>;

    #[doc(hidden)]
    fn _get_stride_mut(&mut self) -> &mut SpatialStride<QI, NUM_DIMS>;
}


pub struct OwningSpatialIndexingHelper<QI, QLinear, QCoord, QDim, const NUM_DIMS: usize>
where
    QI: QuantizedUnsignedIntegerUnwrappedTrait<QuantType=QI>,
    QLinear: QuantizedUnsignedIntegerTrait<QuantType=QI>,
    QCoord: QuantizedUnsignedIntegerTrait<QuantType=QI>,
    QDim: QuantizedUnsignedIntegerTrait<QuantType=QI>,
{
    dimensions: SpatialDimensions<QDim, NUM_DIMS>,
    stride: SpatialStride<QI, NUM_DIMS>,
    _p: core::marker::PhantomData<(QLinear, QCoord)>
}


impl<QI, QLinear, QCoord, QDim, const NUM_DIMS: usize>
OwningSpatialIndexingHelper<QI, QLinear, QCoord, QDim, NUM_DIMS>
where
    QI: QuantizedUnsignedIntegerUnwrappedTrait<QuantType=QI>,
    QLinear: QuantizedUnsignedIntegerTrait<QuantType=QI>,
    QCoord: QuantizedUnsignedIntegerTrait<QuantType=QI>,
    QDim: QuantizedUnsignedIntegerTrait<QuantType=QI>,
{

}

impl<QI, QLinear, QCoord, QDim, const NUM_DIMS: usize, const AXIS_ORDER_U8: u8>
SpatialIndexingHelper<QI, NUM_DIMS, AXIS_ORDER_U8> for
OwningSpatialIndexingHelper<QI, QLinear, QCoord, QDim, NUM_DIMS>
where
    QI: QuantizedUnsignedIntegerUnwrappedTrait<QuantType=QI>,
    QLinear: QuantizedUnsignedIntegerTrait<QuantType=QI>,
    QCoord: QuantizedUnsignedIntegerTrait<QuantType=QI>,
    QDim: QuantizedUnsignedIntegerTrait<QuantType=QI>,
{
    const AXIS_ORDER: AxisOrder<NUM_DIMS> = AxisOrder::from_u8(AXIS_ORDER_U8);

    type LinearIndex = QLinear;
    type CoordinateQuant = QCoord;
    type DimensionsQuant = QDim;

    fn get_dimensions(&self) -> &SpatialDimensions<Self::DimensionsQuant, NUM_DIMS> {
        &self.dimensions
    }

    fn _get_stride(&self) -> &SpatialStride<QI, NUM_DIMS> {
        &self.stride
    }

    fn _get_dimensions_mut(&mut self) -> &mut SpatialDimensions<Self::DimensionsQuant, NUM_DIMS> {
        &mut self.dimensions
    }

    fn _get_stride_mut(&mut self) -> &mut SpatialStride<QI, NUM_DIMS> {
        &mut self.stride
    }
}