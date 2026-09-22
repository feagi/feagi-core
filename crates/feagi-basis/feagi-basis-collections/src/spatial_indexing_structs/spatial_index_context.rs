
use feagi_basis_quantization::prelude::QuantizedUnsignedIntegerTrait;
use crate::prelude::{SpatialCoordinate, SpatialDimensions};
use crate::spatial_indexing_structs::axis_order::{AxisOrderArray, AxisOrderEnum};
use crate::spatial_indexing_structs::SpatialStride;

#[derive(Clone, Debug, Hash, Eq, PartialEq, ::serde::Serialize)]
#[serde(bound(
    serialize = "QDims: ::serde::Serialize",
))]
pub struct SpatialContext<QDims, const NUM_DIMS: usize>
where
    QDims: QuantizedUnsignedIntegerTrait,
{
    dimensions: SpatialDimensions<QDims, NUM_DIMS>,
    axis_order: AxisOrderArray<NUM_DIMS>,
    #[serde(skip)]
    stride: SpatialStride<NUM_DIMS>,
}

impl<QDims, const NUM_DIMS: usize> SpatialContext<QDims, NUM_DIMS>
where
    QDims: QuantizedUnsignedIntegerTrait
{

    pub fn new(dimensions: SpatialDimensions<QDims, NUM_DIMS>,
               axis_order: AxisOrderEnum) -> Self {
        let axis_order = AxisOrderEnum::generate_axis_order_array(axis_order);
        let stride = SpatialStride::new_stride(&dimensions, &axis_order);
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
struct SpatialContextDe<QDims, const NUM_DIMS: usize>
where
    QDims: QuantizedUnsignedIntegerTrait,
{
    dimensions: SpatialDimensions<QDims, NUM_DIMS>,
    axis_order: AxisOrderArray<NUM_DIMS>,
}
impl<'de, QDims, const NUM_DIMS: usize> ::serde::Deserialize<'de>
for SpatialContext<QDims, NUM_DIMS>
where
    QDims: QuantizedUnsignedIntegerTrait,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        let de = SpatialContextDe::<QDims, NUM_DIMS>::deserialize(deserializer)?;
        Ok(Self {
            dimensions: de.dimensions,
            axis_order: de.axis_order,
            stride: SpatialStride::new_stride(&de.dimensions, &de.axis_order),
        })
    }
}