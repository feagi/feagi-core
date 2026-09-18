use feagi_basis_quantization::prelude::QuantizedUnsignedIntegerTrait;
use crate::feagi_collection_error::FeagiDataCollectionError;
use crate::spatial_indexing_structs::base_shared::SpatialIndexingBase;

macro_rules! generate_quantized_coordinate {
    ($(#[$doc:meta])* $name:ident, $size:expr) => {
        generate_quantized_coordinate!(@def $(#[$doc])* , $name, $size;);
    };
    ($(#[$doc:meta])* $vis:vis, $name:ident, $size:expr) => {
        generate_quantized_coordinate!(@def $(#[$doc])* $vis, $name, $size;);
    };

    (@def $(#[$doc:meta])* $vis:vis, $name:ident, $size:expr;) => {
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

        impl<'de, QI: ::feagi_basis_quantization::prelude::QuantizedUnsignedIntegerTrait> SpatialIndexingCoordinate<'de, QI, $size> for $name<QI> {
            const ZEROS: Self = Self {
                data: [QI::QUANT_ZERO; $size],
            };
        }
    };
}

/// Defines a spatial coordinate in higher dimensional space
pub trait SpatialIndexingCoordinate<'de, QI: QuantizedUnsignedIntegerTrait, const NUM_DIMS: usize>:
    SpatialIndexingBase<'de, QI, NUM_DIMS>
{
    /// Coordinate with all elements as zero.
    const ZEROS: Self;

    /// Create a new coordinate
    fn new_coordinate(data: [QI; NUM_DIMS]) -> Self {
        unsafe { Self::new(data).unwrap_unchecked() } // We know this will always be OK
    }

    /// Get references to elements
    fn as_coordinate_slice(&self) -> &[QI; NUM_DIMS] {
        self.as_slice()
    }

    /// Get mutable references to elements
    fn as_coordinate_mut_slice(&mut self) -> &mut [QI; NUM_DIMS] {
        self.as_mut_slice()
    }
}
