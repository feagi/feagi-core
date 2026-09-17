use feagi_basis_quantization::prelude::QuantizedUnsignedIntegerTrait;
use crate::feagi_collection_error::FeagiDataCollectionError;
use crate::spatial_indexing::base_shared::SpatialIndexingBase;
use crate::spatial_indexing::coordinate::SpatialIndexingCoordinate;


/// Defines some higher number of dimensions
pub trait SpatialIndexingDimensions<'de, QI: QuantizedUnsignedIntegerTrait, const NUM_DIMS: usize>:
SpatialIndexingBase<'de, QI, NUM_DIMS> {
    /// The type of coordinate this dimension accepts
    type Coord: SpatialIndexingCoordinate<'de, QI, NUM_DIMS>;

    /// Constructor (no value can be zero)
    fn new(data: [QI; NUM_DIMS]) -> Result<Self, FeagiDataCollectionError>;
}