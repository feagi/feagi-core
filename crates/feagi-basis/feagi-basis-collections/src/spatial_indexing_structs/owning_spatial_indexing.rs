use core::iter::FusedIterator;
use serde::{Serialize, Deserialize};
use feagi_basis_quantization::prelude::QuantizedUnsignedIntegerTrait;
use crate::prelude::SpatialCoordinate;
use crate::spatial_indexing_structs::axis_order::{AxisOrderEnum, AxisOrderArray};
use crate::spatial_indexing_structs::dimensions::SpatialDimensions;
use crate::spatial_indexing_structs::stride::SpatialStride;

struct OwningSpatialIndexingHelper<QLinear, QCoord, QDim, const NUM_DIMS: usize>
where
    QLinear: QuantizedUnsignedIntegerTrait,
    QCoord: QuantizedUnsignedIntegerTrait<QuantType=QLinear::QuantType>,
    QDim: QuantizedUnsignedIntegerTrait<QuantType=QLinear::QuantType>,
{
    dimensions: SpatialDimensions<QDim, NUM_DIMS>,
    axis_order: AxisOrderArray<NUM_DIMS>,
    _p: core::marker::PhantomData<(QLinear, QCoord)>,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Serialize, Deserialize)]
pub struct OwningSpatialIndexing<QLinear, QCoord, QDim, const NUM_DIMS: usize>
where
    QLinear: QuantizedUnsignedIntegerTrait,
    QCoord: QuantizedUnsignedIntegerTrait<QuantType=QLinear::QuantType>,
    QDim: QuantizedUnsignedIntegerTrait<QuantType=QLinear::QuantType>,
{
    dimensions: SpatialDimensions<QDim, NUM_DIMS>,
    axis_order: AxisOrderArray<NUM_DIMS>,
    #[serde(skip)]
    stride: SpatialStride<NUM_DIMS>,
    #[serde(skip)]
    _p: core::marker::PhantomData<(QLinear, QCoord)>,
}

impl<QLinear, QCoord, QDim, const NUM_DIMS: usize> 
OwningSpatialIndexing<QLinear, QCoord, QDim, NUM_DIMS>
where
    QLinear: QuantizedUnsignedIntegerTrait,
    QCoord: QuantizedUnsignedIntegerTrait<QuantType=QLinear::QuantType>,
    QDim: QuantizedUnsignedIntegerTrait<QuantType=QLinear::QuantType>,
{
    /// Builds an owned helper from `dimensions` and the given axis-order enum.
    pub fn from_dimensions(
        dimensions: SpatialDimensions<QDim, NUM_DIMS>,
        axis_order_enum: AxisOrderEnum,
    ) -> Self {
        let axis_order = axis_order_enum.generate_axis_order_array::<NUM_DIMS>();
        
        let stride = SpatialStride::new_stride(
            &dimensions,
            &axis_order,
        );
        Self {
            dimensions,
            axis_order,
            stride,
            _p: core::marker::PhantomData,
        }
    }

    pub fn get_dimensions(&self) -> &SpatialDimensions<QDim, NUM_DIMS> {
        &self.dimensions
    }
    
    pub fn update_dimensions(&mut self, new_dimensions: SpatialDimensions<QDim, NUM_DIMS>) {
        self.dimensions = new_dimensions;
        self.update_stride();
    }

    pub fn linear_to_coordinate(&self, linear: QLinear) -> SpatialCoordinate<QCoord, NUM_DIMS> {
        self.stride.linear_to_coordinate(linear, &self.dimensions)
    }
    
    pub fn coordinate_to_linear(&self, coordinate: &SpatialCoordinate<QCoord, NUM_DIMS>) -> QLinear {
        self.stride.coordinate_to_linear(coordinate)
    }

    /// Iterates all coordinates within the current dimensions, in linear index order.
    fn iter_coordinates(
        &self,
    ) -> SpatialCoordinateIter<'_, QLinear, QCoord, QDim, NUM_DIMS> {
        SpatialCoordinateIter::new(&self.stride, self.get_dimensions())
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
        let stride = self.stride;
        let dimensions = *self.get_dimensions();
        (0..total).into_par_iter().map(move |index| {
            let linear = QLinear::quant_from_usize_unchecked(index);
            stride.linear_to_coordinate::<QLinear, Self::CoordinateQuant, Self::DimensionsQuant>(
                linear,
                &dimensions,
            )
        })
    }
    
    /// If dimensions or axis order changes, run this to update the stride cache
    fn update_stride(&mut self) {
        self.stride = SpatialStride::new_stride(&self.dimensions, &self.axis_order);
    }
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
    stride: &'a SpatialStride<NUM_DIMS>,
    dimensions: &'a SpatialDimensions<QDims, NUM_DIMS>,
    current: usize,
    total: usize,
    _p: core::marker::PhantomData<(QLinear, QCoord)>,
}

impl<
    'a,
    QLinear: QuantizedUnsignedIntegerTrait,
    QCoord: QuantizedUnsignedIntegerTrait<QuantType = QLinear::QuantType>,
    QDims: QuantizedUnsignedIntegerTrait<QuantType = QLinear::QuantType>,
    const NUM_DIMS: usize,
> SpatialCoordinateIter<'a, QLinear, QCoord, QDims, NUM_DIMS> {
    pub(crate) fn new(
        stride: &'a SpatialStride<NUM_DIMS>,
        dimensions: &'a SpatialDimensions<QDims, NUM_DIMS>,
    ) -> Self {
        let total = dimensions.spatial_element_count();
        Self {
            stride,
            dimensions,
            current: 0,
            total,
            _p: core::marker::PhantomData,
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
