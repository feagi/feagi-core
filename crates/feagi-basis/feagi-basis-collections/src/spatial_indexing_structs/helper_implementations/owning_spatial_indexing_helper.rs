use feagi_basis_quantization::prelude::QuantizedUnsignedIntegerTrait;
use crate::spatial_indexing_structs::axis_order::{AxisOrder, AxisOrderIdentifier};
use crate::spatial_indexing_structs::dimensions::SpatialDimensions;
use crate::spatial_indexing_structs::spatial_indexing_helpers::{SpatialIndexingHelper, SpatialIndexingHelperMut};
use crate::spatial_indexing_structs::stride::SpatialStride;

/// An index coordinate converter that owns all the data, for ease of portability
#[derive(Debug, ::serde::Serialize, ::serde::Deserialize)]
#[serde(bound(
    serialize = "SpatialDimensions<QDim, NUM_DIMS>: ::serde::Serialize, QLinear::QuantType: ::serde::Serialize",
    deserialize = "SpatialDimensions<QDim, NUM_DIMS>: ::serde::Deserialize<'de>, QLinear::QuantType: ::serde::Deserialize<'de>"
))]
pub struct OwningSpatialIndexingHelper<QLinear, QCoord, QDim, const NUM_DIMS: usize>
where
    QLinear: QuantizedUnsignedIntegerTrait,
    QCoord: QuantizedUnsignedIntegerTrait<QuantType=QLinear::QuantType>,
    QDim: QuantizedUnsignedIntegerTrait<QuantType=QLinear::QuantType>,
{
    dimensions: SpatialDimensions<QDim, NUM_DIMS>,
    stride: SpatialStride<QLinear::QuantType, NUM_DIMS>,
    #[serde(skip)]
    _p: core::marker::PhantomData<(QLinear, QCoord)>,
}

impl<QLinear, QCoord, QDim, const NUM_DIMS: usize>
    OwningSpatialIndexingHelper<QLinear, QCoord, QDim, NUM_DIMS>
where
    QLinear: QuantizedUnsignedIntegerTrait,
    QCoord: QuantizedUnsignedIntegerTrait<QuantType = QLinear::QuantType>,
    QDim: QuantizedUnsignedIntegerTrait<QuantType = QLinear::QuantType>,
{
    /// Builds an owned helper from `dimensions` and the given axis-order identifier.
    pub fn from_dimensions(
        dimensions: SpatialDimensions<QDim, NUM_DIMS>,
        axis_order_identifier: AxisOrderIdentifier,
    ) -> Self {
        let axis_order = AxisOrder::<NUM_DIMS>::from_identifier(axis_order_identifier);
        let stride = SpatialStride::new_stride(
            &dimensions,
            &axis_order.generate_axis_order_array(),
        );
        Self {
            dimensions,
            stride,
            _p: core::marker::PhantomData,
        }
    }
}

impl<
    QLinear,
    QCoord,
    QDim,
    const NUM_DIMS: usize,
    const AXIS_ORDER_IDENTIFIER: AxisOrderIdentifier,
> SpatialIndexingHelper<QLinear, NUM_DIMS, AXIS_ORDER_IDENTIFIER> for
    OwningSpatialIndexingHelper<QLinear, QCoord, QDim, NUM_DIMS>
where
    QLinear: QuantizedUnsignedIntegerTrait,
    QCoord: QuantizedUnsignedIntegerTrait<QuantType=QLinear::QuantType>,
    QDim: QuantizedUnsignedIntegerTrait<QuantType=QLinear::QuantType>,
{
    const AXIS_ORDER: AxisOrder<NUM_DIMS> =
        AxisOrder::from_identifier(AXIS_ORDER_IDENTIFIER);
    
    type CoordinateQuant = QCoord;
    type DimensionsQuant = QDim;

    fn get_dimensions(&self) -> &SpatialDimensions<Self::DimensionsQuant, NUM_DIMS> {
        &self.dimensions
    }

    fn _get_stride(&self) -> &SpatialStride<QLinear::QuantType, NUM_DIMS> {
        &self.stride
    }
}

impl<
    QLinear,
    QCoord,
    QDim,
    const NUM_DIMS: usize,
    const AXIS_ORDER_IDENTIFIER: AxisOrderIdentifier,
> SpatialIndexingHelperMut<QLinear, NUM_DIMS, AXIS_ORDER_IDENTIFIER> for
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