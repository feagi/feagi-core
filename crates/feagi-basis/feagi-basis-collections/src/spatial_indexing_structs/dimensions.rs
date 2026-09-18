use feagi_basis_quantization::prelude::QuantizedUnsignedIntegerTrait;
use crate::feagi_collection_error::{FeagiDataCollectionError, FeagiFailInvalidDimensions};
use crate::spatial_indexing_structs::base_shared::SpatialIndexingBase;
use crate::spatial_indexing_structs::coordinate::SpatialIndexingCoordinate;

macro_rules! generate_quantized_dimension {
    ($(#[$doc:meta])* $name:ident, $size:expr, $coord_type:ident) => {
        generate_quantized_dimension!(@def $(#[$doc])* , $name, $size, $coord_type;);
    };
    ($(#[$doc:meta])* $vis:vis, $name:ident, $size:expr, $coord_type:ident) => {
        generate_quantized_dimension!(@def $(#[$doc])* $vis, $name, $size, $coord_type;);
    };

    (@def $(#[$doc:meta])* $vis:vis, $name:ident, $size:expr, $coord_type:ident;) => {
        $(#[$doc])*
        #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, ::serde::Serialize, ::serde::Deserialize)]
        $vis struct $name<QI: ::feagi_basis_quantization::prelude::QuantizedUnsignedIntegerTrait> {
            #[serde(with = "serde_arrays")]
            data: [QI; $size],
        }

        impl<'de, QI: ::feagi_basis_quantization::prelude::QuantizedUnsignedIntegerTrait> SpatialIndexingBase<'de, QI, $size> for $name<QI> {
            fn new(data: [QI; $size]) -> Result<Self, FeagiDataCollectionError> {
                Ok(Self { data })
            }

            fn as_slice(&self) -> &[QI; $size] {
                &self.data
            }

            fn as_mut_slice(&mut self) -> &mut [QI; $size] {
                &mut self.data
            }
        }

        impl<'de, QI: ::feagi_basis_quantization::prelude::QuantizedUnsignedIntegerTrait> SpatialIndexingDimensions<'de, QI, $size> for $name<QI> {
            type Coord = $coord_type<QI>;

            fn new_dimensions(data: [QI; $size]) -> Result<Self, FeagiDataCollectionError> {
                if data.contains(&QI::QUANT_ZERO) {
                    return Err(FeagiFailInvalidDimensions::new("Dimensions cannot be 0 in any direction!").into())
                }
                Ok(Self {data})
            }
        }
    };
}


/// Defines some higher number of dimensions
pub trait SpatialIndexingDimensions<'de, QI: QuantizedUnsignedIntegerTrait, const NUM_DIMS: usize>:
SpatialIndexingBase<'de, QI, NUM_DIMS> {
    /// The type of coordinate this dimension accepts
    type Coord: SpatialIndexingCoordinate<'de, QI, NUM_DIMS>;

    /// Constructor (no value can be zero)
    fn new_dimensions(data: [QI; NUM_DIMS]) -> Result<Self, FeagiDataCollectionError>;
}