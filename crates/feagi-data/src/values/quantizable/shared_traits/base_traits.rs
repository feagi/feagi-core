use half::f16;
use crate::values::quantizable::custom_data_types::StorageF8;
use crate::values::quantizable::QuantizationLevel;
use crate::values::quantizable::shared_traits::SupportsBasicCoreMathOps;

/// Common base for all quantizable types
#[doc(hidden)]
pub trait QuantizedElementBase:
SupportsBasicCoreMathOps
{
    const QUANTIZATION_LEVEL: QuantizationLevel;
    const QUANT_ZERO: Self;
}

impl QuantizedElementBase for usize{
    // Sizing can vary depending on build type
    #[cfg(target_pointer_width = "64")]
    const QUANTIZATION_LEVEL: QuantizationLevel = QuantizationLevel::Bit64;
    #[cfg(target_pointer_width = "32")]
    const QUANTIZATION_LEVEL: QuantizationLevel = QuantizationLevel::Bit32;
    const QUANT_ZERO: Self = 0;
}

impl QuantizedElementBase for u8 {
    const QUANTIZATION_LEVEL: QuantizationLevel = QuantizationLevel::Bit8;
    const QUANT_ZERO: Self = 0;
}

impl QuantizedElementBase for u16 {
    const QUANTIZATION_LEVEL: QuantizationLevel = QuantizationLevel::Bit16;
    const QUANT_ZERO: Self = 0;
}

impl QuantizedElementBase for u32 {
    const QUANTIZATION_LEVEL: QuantizationLevel = QuantizationLevel::Bit32;
    const QUANT_ZERO: Self = 0;
}

impl QuantizedElementBase for u64 {
    const QUANTIZATION_LEVEL: QuantizationLevel = QuantizationLevel::Bit64;
    const QUANT_ZERO: Self = 0;
}

// Lol no we are not doing u128 or i128


impl QuantizedElementBase for isize{
    // Sizing can vary depending on build type
    #[cfg(target_pointer_width = "64")]
    const QUANTIZATION_LEVEL: QuantizationLevel = QuantizationLevel::Bit64;
    #[cfg(target_pointer_width = "32")]
    const QUANTIZATION_LEVEL: QuantizationLevel = QuantizationLevel::Bit32;
    const QUANT_ZERO: Self = 0;
}

impl QuantizedElementBase for i8 {
    const QUANTIZATION_LEVEL: QuantizationLevel = QuantizationLevel::Bit8;
    const QUANT_ZERO: Self = 0;
}

impl QuantizedElementBase for i16 {
    const QUANTIZATION_LEVEL: QuantizationLevel = QuantizationLevel::Bit16;
    const QUANT_ZERO: Self = 0;
}

impl QuantizedElementBase for i32 {
    const QUANTIZATION_LEVEL: QuantizationLevel = QuantizationLevel::Bit32;
    const QUANT_ZERO: Self = 0;
}

impl QuantizedElementBase for i64 {
    const QUANTIZATION_LEVEL: QuantizationLevel = QuantizationLevel::Bit64;
    const QUANT_ZERO: Self = 0;
}

// A bad choice for computation
impl QuantizedElementBase for StorageF8 {
    const QUANTIZATION_LEVEL: QuantizationLevel = QuantizationLevel::Bit8;
    const QUANT_ZERO: Self = StorageF8::ZERO;
}

impl QuantizedElementBase for f16 {
    const QUANTIZATION_LEVEL: QuantizationLevel = QuantizationLevel::Bit16;
    const QUANT_ZERO: Self = f16::ZERO;
}

impl QuantizedElementBase for f32 {
    const QUANTIZATION_LEVEL: QuantizationLevel = QuantizationLevel::Bit32;
    const QUANT_ZERO: Self = 0.0;
}

impl QuantizedElementBase for f64 {
    const QUANTIZATION_LEVEL: QuantizationLevel = QuantizationLevel::Bit64;
    const QUANT_ZERO: Self = 0.0;
}
