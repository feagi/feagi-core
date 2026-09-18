use feagi_basis_quantization::prelude::QuantizedUnsignedIntegerTrait;
use crate::feagi_collection_error::{FeagiDataCollectionError, FeagiFailInvalidAxisOrder};

/// Generic owned axis-order value for an N-dimensional index space.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, ::serde::Serialize, ::serde::Deserialize)]
pub struct SpatialAxisOrder<QI: QuantizedUnsignedIntegerTrait, const NUM_DIMS: usize> {
    #[serde(with = "serde_arrays")]
    data: [QI; NUM_DIMS],
}

impl<QI: QuantizedUnsignedIntegerTrait, const NUM_DIMS: usize> SpatialAxisOrder<QI, NUM_DIMS> {
    /// Constructor.
    pub fn new_axis_order(data: [QI; NUM_DIMS]) -> Result<Self, FeagiDataCollectionError> {
        // Axis order must be a permutation of [0, 1, ..., NUM_DIMS - 1].
        let mut seen = [false; NUM_DIMS];
        for axis in data.iter() {
            let axis_usize = axis.quant_to_usize();
            if axis_usize >= NUM_DIMS {
                return Err(
                    FeagiFailInvalidAxisOrder::new("Axis order contains out-of-range axis index.")
                        .into(),
                );
            }
            if seen[axis_usize] {
                return Err(
                    FeagiFailInvalidAxisOrder::new("Axis order contains a repeated axis index.")
                        .into(),
                );
            }
            seen[axis_usize] = true;
        }
        Ok(Self { data })
    }

    /// Borrow axis order as a fixed-size slice.
    pub fn as_slice(&self) -> &[QI; NUM_DIMS] {
        &self.data
    }

    /// Mutably borrow axis order as a fixed-size slice.
    pub fn as_mut_slice(&mut self) -> &mut [QI; NUM_DIMS] {
        &mut self.data
    }
}


impl<QI: QuantizedUnsignedIntegerTrait, const NUM_DIMS: usize> Default for SpatialAxisOrder<QI, NUM_DIMS> {
    fn default() -> Self {
        let mut out = [QI::QUANT_ZERO; NUM_DIMS];
        let mut x = QI::QUANT_ZERO;
        for i in 0..NUM_DIMS {
            out[i] = x;
            x += QI::QUANT_ONE;
        };
        SpatialAxisOrder { data: out }
    }
}
