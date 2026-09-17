use feagi_basis_quantization::prelude::QuantizedUnsignedIntegerTrait;
use crate::feagi_collection_error::FeagiDataCollectionError;
use crate::spatial_indexing_structs::base_shared::SpatialIndexingBase;

// TODO use this!

macro_rules! generate_quantized_stride {
    ($(#[$doc:meta])* $name:ident, $size:expr) => {
        generate_quantized_coordinate!(@def $(#[$doc])* , $name, $size;);
    };
    ($(#[$doc:meta])* $vis:vis, $name:ident, $size:expr) => {
        generate_quantized_coordinate!(@def $(#[$doc])* $vis, $name, $size;);
    };

    (@def $(#[$doc:meta])* $vis:vis, $name:ident, $size:expr;) => {
        $(#[$doc])*
        #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
        $vis struct $name<QI: QuantizedUnsignedIntegerTrait> {
            #[serde(with = "serde_arrays")]
            data: [QI; $size],
        }

        impl<'de, QI: QuantizedUnsignedIntegerTrait> SpatialIndexingBase<'de, QI, $size> for $name<QI> {
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

        impl<'de, QI: QuantizedUnsignedIntegerTrait> SpatialIndexingStride<'de, QI, $size> for $name<QI> {

        }
    };
}

/// Defines a spatial stride in higher dimensional space
pub trait SpatialIndexingStride<'de, QI: QuantizedUnsignedIntegerTrait, const NUM_DIMS: usize>:
SpatialIndexingBase<'de, QI, NUM_DIMS> {

    /// A default incrementing x -> y -> z... stride
    const DEFAULT_STRIDE: Self;

    /// Constructor for some custom stride
    fn new_custom(data: [QI; NUM_DIMS]) -> Result<Self, FeagiDataCollectionError>;

}