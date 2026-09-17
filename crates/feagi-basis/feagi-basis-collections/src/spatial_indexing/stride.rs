use feagi_basis_quantization::prelude::QuantizedUnsignedIntegerTrait;
use crate::feagi_collection_error::FeagiDataCollectionError;
use crate::spatial_indexing::base_shared::SpatialIndexingBase;

/// Defines a spatial stride in higher dimensional space
pub trait SpatialIndexingStride<'de, QI: QuantizedUnsignedIntegerTrait, const NUM_DIMS: usize>:
SpatialIndexingBase<'de, QI, NUM_DIMS> {
    /// Constructor for some custom stride
    fn new_custom(data: [QI; NUM_DIMS]) -> Result<Self, FeagiDataCollectionError>;

    /// Default incrementing X -> y -> z stride
    fn new_default() -> Self;
}
