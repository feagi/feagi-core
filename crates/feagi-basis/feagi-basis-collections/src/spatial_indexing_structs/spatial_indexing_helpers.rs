use crate::spatial_indexing_structs::coordinate::SpatialCoordinate;
use crate::spatial_indexing_structs::dimensions::SpatialDimensions;
use crate::spatial_indexing_structs::stride::SpatialStride;
use feagi_basis_quantization::prelude::{QuantizedUnsignedIntegerTrait, QuantizedUnsignedIntegerUnwrappedTrait};
use crate::spatial_indexing_structs::axis_order::{AxisOrder, AxisOrderIdentifier};

/// A trait to easily group dimensions, stride, and axis order to handle linear coordinate conversions
pub trait SpatialIndexingHelper<
    QLinear: QuantizedUnsignedIntegerTrait,
    const NUM_DIMS: usize,
    const AXIS_ORDER_IDENTIFIER: AxisOrderIdentifier
> {
    const AXIS_ORDER: AxisOrder<NUM_DIMS> = AxisOrder::from_identifier(AXIS_ORDER_IDENTIFIER);

    type CoordinateQuant: QuantizedUnsignedIntegerTrait<QuantType=QLinear::QuantType>;

    type DimensionsQuant: QuantizedUnsignedIntegerTrait<QuantType=QLinear::QuantType>;

    /// Get the current in use dimensions
    fn get_dimensions(&self) -> &SpatialDimensions<Self::DimensionsQuant, NUM_DIMS>;

    fn coordinate_to_linear(&self, coordinate: &SpatialCoordinate<Self::CoordinateQuant, NUM_DIMS>) -> QLinear {
        let stride = self._get_stride();
        stride.coordinate_to_linear::<QLinear, Self::CoordinateQuant>(coordinate)
    }

    fn linear_to_coordinate(&self, linear_index: QLinear) -> SpatialCoordinate<Self::CoordinateQuant, NUM_DIMS> {
        let stride = self._get_stride();
        stride.linear_to_coordinate::<QLinear, Self::CoordinateQuant, Self::DimensionsQuant>(linear_index, self.get_dimensions())
    }

    // NOTE: Yes, this is a bit ugly here, but the best way above to handle this that I can think of right now

    #[doc(hidden)]
    fn _get_stride(&self) -> &SpatialStride<QLinear::QuantType, NUM_DIMS>;

}

/// A trait to easily group dimensions, stride, and axis order to handle linear coordinate
/// conversions, AS WELL as modifying the dimensions of a given struct
pub trait SpatialIndexingHelperMut<
    QLinear: QuantizedUnsignedIntegerTrait,
    const NUM_DIMS: usize,
    const AXIS_ORDER_IDENTIFIER: AxisOrderIdentifier
>: SpatialIndexingHelper<QLinear, NUM_DIMS, AXIS_ORDER_IDENTIFIER>
{
    /// Update dimensions, including updating the stride as well
    fn update_dimensions(&mut self, new_dims: SpatialDimensions<Self::DimensionsQuant, NUM_DIMS>) {
        self._get_stride_mut().update_stride(&new_dims, &Self::AXIS_ORDER.generate_axis_order_array());
        *self._get_dimensions_mut() = new_dims;
    }

    // NOTE: Yes, this is a bit ugly here, but the best way above to handle this that I can think of right now

    #[doc(hidden)]
    fn _get_dimensions_mut(&mut self) -> &mut SpatialDimensions<Self::DimensionsQuant, NUM_DIMS>;

    #[doc(hidden)]
    fn _get_stride_mut(&mut self) -> &mut SpatialStride<QLinear::QuantType, NUM_DIMS>;
}