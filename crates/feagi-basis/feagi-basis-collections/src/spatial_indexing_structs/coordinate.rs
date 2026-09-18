use feagi_basis_quantization::prelude::QuantizedUnsignedIntegerTrait;

/// Generic owned coordinate for an N-dimensional index space.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, ::serde::Serialize, ::serde::Deserialize)]
pub struct SpatialCoordinate<QI: QuantizedUnsignedIntegerTrait, const NUM_DIMS: usize> {
    #[serde(with = "serde_arrays")]
    data: [QI; NUM_DIMS],
}

impl<QI: QuantizedUnsignedIntegerTrait, const NUM_DIMS: usize> SpatialCoordinate<QI, NUM_DIMS> {
    /// Coordinate with all elements as zero.
    pub const ZEROS: Self = Self {
        data: [QI::QUANT_ZERO; NUM_DIMS],
    };

    /// Create a new coordinate.
    pub fn new_coordinate(data: [QI; NUM_DIMS]) -> Self {
        Self { data }
    }

    /// Borrow coordinate entries as a fixed-size slice.
    pub fn as_slice(&self) -> &[QI; NUM_DIMS] {
        &self.data
    }

    /// Mutably borrow coordinate entries as a fixed-size slice.
    pub fn as_mut_slice(&mut self) -> &mut [QI; NUM_DIMS] {
        &mut self.data
    }
}
