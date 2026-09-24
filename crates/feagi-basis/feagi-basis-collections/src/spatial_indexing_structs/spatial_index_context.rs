use std::marker::PhantomData;
use feagi_basis_quantization::prelude::QuantizedUnsignedIntegerTrait;
use crate::prelude::{SpatialCoordinate, SpatialDimensions};
use crate::spatial_indexing_structs::axis_order::{AxisOrder, AxisOrderArray};
use crate::spatial_indexing_structs::SpatialStride;

#[derive(Clone, Debug, Hash, Eq, PartialEq, ::serde::Serialize)]
#[serde(bound(
    serialize = "QDims: ::serde::Serialize",
))]
pub struct SpatialContext<QDims, AxisOrderType, const NUM_DIMS: usize>
where
    QDims: QuantizedUnsignedIntegerTrait,
    AxisOrderType: AxisOrder<NUM_DIMS>,
{
    dimensions: SpatialDimensions<QDims, NUM_DIMS>,
    axis_order: AxisOrderArray<NUM_DIMS>,
    #[serde(skip)]
    stride: SpatialStride<NUM_DIMS>,
    #[serde(skip)]
    _p: core::marker::PhantomData<AxisOrderType>
}

impl<QDims, AxisOrderType, const NUM_DIMS: usize> SpatialContext<QDims, AxisOrderType, NUM_DIMS>
where
    QDims: QuantizedUnsignedIntegerTrait,
    AxisOrderType: AxisOrder<NUM_DIMS>,
{
    pub fn new(dimensions: SpatialDimensions<QDims, NUM_DIMS>) -> Self {
        let axis_order = AxisOrderType::AXIS_ORDER_ARRAY;
        let stride = SpatialStride::new_stride(&dimensions, &axis_order);
        Self {
            dimensions, axis_order, stride, _p: PhantomData
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
        self.stride.linear_to_coordinate(linear_index, &self.dimensions)
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
    
    pub fn get_dimensions(&self) -> &SpatialDimensions<QDims, NUM_DIMS> {
        &self.dimensions
    }
    
    pub fn get_axis_order_array(&self) -> &AxisOrderArray<NUM_DIMS> {
        &self.axis_order
    }
}

/// just used for deserializing and initing the stride again
#[derive(::serde::Deserialize)]
#[serde(bound(deserialize = "QDims: ::serde::de::DeserializeOwned"))]
struct SpatialContextDe<QDims, AxisOrderType, const NUM_DIMS: usize>
where
    QDims: QuantizedUnsignedIntegerTrait,
    AxisOrderType: AxisOrder<NUM_DIMS>,
{
    dimensions: SpatialDimensions<QDims, NUM_DIMS>,
    #[serde(skip)]
    _p: PhantomData<AxisOrderType>
}
impl<'de, QDims, AxisOrderType, const NUM_DIMS: usize> ::serde::Deserialize<'de>
for SpatialContext<QDims, AxisOrderType, NUM_DIMS>
where
    QDims: QuantizedUnsignedIntegerTrait,
    AxisOrderType: AxisOrder<NUM_DIMS>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        let de = SpatialContextDe::<QDims, AxisOrderType, NUM_DIMS>::deserialize(deserializer)?;
        let axis_order = AxisOrderType::AXIS_ORDER_ARRAY;
        Ok(Self {
            dimensions: de.dimensions,
            axis_order,
            stride: SpatialStride::new_stride(&de.dimensions, &axis_order),
            _p: Default::default(),
        })
    }
}