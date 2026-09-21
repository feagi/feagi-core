use crate::feagi_collection_error::{FeagiDataCollectionError, FeagiFailInvalidDimensions};
use feagi_basis_quantization::prelude::QuantizedUnsignedIntegerTrait;

/// Generic owned dimensions value for an N-dimensional index space.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, ::serde::Serialize, ::serde::Deserialize)]
pub struct SpatialDimensions<QI: QuantizedUnsignedIntegerTrait, const NUM_DIMS: usize> {
    #[serde(with = "serde_arrays")]
    data: [QI; NUM_DIMS],
}

impl<QI: QuantizedUnsignedIntegerTrait, const NUM_DIMS: usize> SpatialDimensions<QI, NUM_DIMS> {
    /// Constructor for dimensions; no axis may be zero.
    pub fn new_dimensions(data: [QI; NUM_DIMS]) -> Result<Self, FeagiDataCollectionError> {
        if data.contains(&QI::QUANT_ZERO) {
            return Err(FeagiFailInvalidDimensions::new("Dimensions cannot be 0 in any direction!").into());
        }
        Ok(Self { data })
    }

    /// Borrow dimensions as a fixed-size slice.
    pub fn as_slice(&self) -> &[QI; NUM_DIMS] {
        &self.data
    }

    /// Mutably borrow dimensions as a fixed-size slice.
    pub fn as_mut_slice(&mut self) -> &mut [QI; NUM_DIMS] {
        &mut self.data
    }

    /// Total number of elements in the index space (product of all axis lengths).
    pub fn spatial_element_count(&self) -> usize {
        self.data.iter().map(|dim| dim.quant_to_usize()).product()
    }
}