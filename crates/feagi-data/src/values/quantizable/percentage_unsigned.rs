use crate::values::quantizable::feagi_data_value_quantization_error::{FeagiDataValueQuantizationError, FeagiFailPercentageOutOfRange};
use crate::values::quantizable::QuantizedDecimalTrait;

/// Internally uses a Quantized Decimal, But exposes methods to have this act as a percentage from
/// 0 - 100 % (0.0 - 1.0)
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Copy)]
pub struct PercentageUnsigned<D: QuantizedDecimalTrait>(D);

impl<D: QuantizedDecimalTrait> PercentageUnsigned<D> {
    const ZERO_PERCENT: Self = Self(D::QUANT_ZERO);
    const HUNDRED_PERCENT: Self = Self(D::QUANT_ONE);

    /// Checks value is between 0.0 - 1.0 before creating itself as such
    pub fn new_checked(value: D) -> Result<Self, FeagiDataValueQuantizationError> {
        if value < D::QUANT_ZERO || value > D::QUANT_ONE {
            return Err(FeagiFailPercentageOutOfRange::new("Attempted to store out of range percentage!", value.quant_to_f32()).into());
        }
        Ok(Self(value))
    }

    /// Enforces value is within range before returning
    pub fn new_clamped(value: D) -> Self {
        Self(value.quant_clamp(D::QUANT_ZERO, D::QUANT_ONE))
    }

    /// Creates percentage without checking if the value is within 0.0 - 1.0. Faster, but risks
    /// undefined behavior if used incorrectly!
    pub fn new_unchecked(value: D) -> Self {
        debug_assert!(
            value >= D::QUANT_ZERO && value <= D::QUANT_ONE,
            "Attempted to store out of range percentage!"
        );
        Self(value)
    }
    
    pub fn from_quantization<FromQuant: QuantizedDecimalTrait>(value: PercentageUnsigned<FromQuant>) -> Self {
        Self(value.get_decimal().to_quantization::<D>())
    }
    
    pub fn to_quantization<ToQuant: QuantizedDecimalTrait>(self) -> PercentageUnsigned<ToQuant> {
        PercentageUnsigned(self.0.to_quantization::<ToQuant>())
    }

    /// Returns the inner 0 - 1.0 decimal contained
    pub fn get_decimal(self) -> D {
        self.0
    }
}

impl<D: QuantizedDecimalTrait> Default for PercentageUnsigned<D> {
    fn default() -> Self {
        Self::ZERO_PERCENT
    }
}

//TODO  impl core::ops::Mul, DIV, MulAssign, DivAssign. Not add / subtract as that easily gets us out of range!
