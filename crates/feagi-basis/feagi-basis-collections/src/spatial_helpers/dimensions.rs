use super::SpatialHelperError;
use feagi_basis_quantization::prelude::QuantizedUnsignedIntegerTrait;
use crate::spatial_context::spatial_context_error::SpatialContextError;

/// Generic owned dimensions value for an N-dimensional index space.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, ::serde::Serialize, ::serde::Deserialize)]
pub struct SpatialDimensions<QI: QuantizedUnsignedIntegerTrait, const NUM_DIMS: usize> {
    #[serde(with = "serde_arrays")]
    data: [QI; NUM_DIMS],
}

impl<QI: QuantizedUnsignedIntegerTrait, const NUM_DIMS: usize> SpatialDimensions<QI, NUM_DIMS> {
    /// Constructor for dimensions; no axis may be zero.
    pub fn new_dimensions(data: [QI; NUM_DIMS]) -> Result<Self, SpatialHelperError> {
        if data.contains(&QI::QUANT_ZERO) {
            return Err(SpatialHelperError::InvalidContext(SpatialContextError::DimensionsCannotBeZero));
        }
        Ok(Self { data })
    }
    
    pub fn new_dimensions_from_usize(data: [usize; NUM_DIMS]) -> Result<Self, SpatialHelperError> {
        let data_in: [QI; NUM_DIMS] = data.map(|e| QI::quant_try_from_usize(e).unwrap()); // TODO Error Handling
        Self::new_dimensions(data_in)        
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