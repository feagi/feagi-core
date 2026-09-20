use core::iter::FusedIterator;
use core::marker::PhantomData;

use crate::spatial_indexing_structs::axis_order::{AxisOrder, AxisOrderIdentifier};
use crate::spatial_indexing_structs::coordinate::SpatialCoordinate;
use crate::spatial_indexing_structs::dimensions::SpatialDimensions;
use crate::spatial_indexing_structs::stride::SpatialStride;
use feagi_basis_quantization::prelude::QuantizedUnsignedIntegerTrait;

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

    /// Iterates all coordinates within the current dimensions, in linear index order.
    fn iter_coordinates(
        &self,
    ) -> SpatialCoordinateIter<'_, QLinear, Self::CoordinateQuant, Self::DimensionsQuant, NUM_DIMS> {
        SpatialCoordinateIter::new(self._get_stride(), self.get_dimensions())
    }

    /// Parallel iterator over all coordinates within the current dimensions.
    #[cfg(feature = "expose_rayon")]
    fn rayon_iter_coordinates(
        &self,
    ) -> rayon::iter::Map<
        rayon::range::Iter<usize>,
        impl Fn(usize) -> SpatialCoordinate<Self::CoordinateQuant, NUM_DIMS> + Send + Sync,
    >
    where
        QLinear: Send,
        Self::CoordinateQuant: Send,
    {
        use rayon::prelude::*;
        let total = self.get_dimensions().spatial_element_count();
        let stride = *self._get_stride();
        let dimensions = *self.get_dimensions();
        (0..total).into_par_iter().map(move |index| {
            let linear = QLinear::quant_from_usize_unchecked(index);
            stride.linear_to_coordinate::<QLinear, Self::CoordinateQuant, Self::DimensionsQuant>(
                linear,
                &dimensions,
            )
        })
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

//region Iterator
/// Iterates every coordinate in a dimension box, in linear index order.
pub struct SpatialCoordinateIter<
    'a,
    QLinear: QuantizedUnsignedIntegerTrait,
    QCoord: QuantizedUnsignedIntegerTrait<QuantType = QLinear::QuantType>,
    QDims: QuantizedUnsignedIntegerTrait<QuantType = QLinear::QuantType>,
    const NUM_DIMS: usize,
> {
    stride: &'a SpatialStride<QLinear::QuantType, NUM_DIMS>,
    dimensions: &'a SpatialDimensions<QDims, NUM_DIMS>,
    current: usize,
    total: usize,
    _p: PhantomData<(QLinear, QCoord)>,
}

impl<
    'a,
    QLinear: QuantizedUnsignedIntegerTrait,
    QCoord: QuantizedUnsignedIntegerTrait<QuantType = QLinear::QuantType>,
    QDims: QuantizedUnsignedIntegerTrait<QuantType = QLinear::QuantType>,
    const NUM_DIMS: usize,
> SpatialCoordinateIter<'a, QLinear, QCoord, QDims, NUM_DIMS> {
    pub(crate) fn new(
        stride: &'a SpatialStride<QLinear::QuantType, NUM_DIMS>,
        dimensions: &'a SpatialDimensions<QDims, NUM_DIMS>,
    ) -> Self {
        let total = dimensions.spatial_element_count();
        Self {
            stride,
            dimensions,
            current: 0,
            total,
            _p: PhantomData,
        }
    }
}

impl<
    'a,
    QLinear: QuantizedUnsignedIntegerTrait,
    QCoord: QuantizedUnsignedIntegerTrait<QuantType = QLinear::QuantType>,
    QDims: QuantizedUnsignedIntegerTrait<QuantType = QLinear::QuantType>,
    const NUM_DIMS: usize,
> Iterator for SpatialCoordinateIter<'a, QLinear, QCoord, QDims, NUM_DIMS> {
    type Item = SpatialCoordinate<QCoord, NUM_DIMS>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.total {
            return None;
        }
        let linear = QLinear::quant_from_usize_unchecked(self.current);
        self.current += 1;
        Some(
            self.stride
                .linear_to_coordinate::<QLinear, QCoord, QDims>(linear, self.dimensions),
        )
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.total.saturating_sub(self.current);
        (remaining, Some(remaining))
    }
}

impl<
    'a,
    QLinear: QuantizedUnsignedIntegerTrait,
    QCoord: QuantizedUnsignedIntegerTrait<QuantType = QLinear::QuantType>,
    QDims: QuantizedUnsignedIntegerTrait<QuantType = QLinear::QuantType>,
    const NUM_DIMS: usize,
> ExactSizeIterator for SpatialCoordinateIter<'a, QLinear, QCoord, QDims, NUM_DIMS> {}

impl<
    'a,
    QLinear: QuantizedUnsignedIntegerTrait,
    QCoord: QuantizedUnsignedIntegerTrait<QuantType = QLinear::QuantType>,
    QDims: QuantizedUnsignedIntegerTrait<QuantType = QLinear::QuantType>,
    const NUM_DIMS: usize,
> FusedIterator for SpatialCoordinateIter<'a, QLinear, QCoord, QDims, NUM_DIMS> {}

//endregion