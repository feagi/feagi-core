use feagi_basis_quantization::prelude::QuantizedUnsignedIntegerTrait;
use crate::feagi_collection_error::FeagiDataCollectionError;

/// Base spatial indexing context data. Is locked within the crate to avoid having the mutable method
/// be easily accessible outside to cause problems
pub trait SpatialIndexingBase<'de, QI: QuantizedUnsignedIntegerTrait, const NUM_DIMS: usize>:
core::fmt::Debug + Clone + Copy + core::hash::Hash + PartialEq
+ ::serde::Serialize + ::serde::Deserialize<'de> {

    /// Number of spatial axes represented by this value.
    const NUM_DIMS: usize = NUM_DIMS;

    /// Constructor
    fn new(data: [QI; NUM_DIMS]) -> Result<Self, FeagiDataCollectionError>;

    /// Get as a slice of specific length
    fn as_slice(&self) -> &[QI; NUM_DIMS];

    /// Get as a mut slice of specific length
    fn as_mut_slice(&mut self) -> &mut [QI; NUM_DIMS];
}
