use half::f16;
use crate::quantizable::base_types::decimal::custom_data_types::StorageF8;
use crate::quantizable::quantization_levels::QuantizationLevel;

/// Common base for all quantizable types (Alloc methods enabled)
#[cfg(not(feature = "alloc"))]
pub trait QuantizedBaseTrait:
Copy
+ Clone
+ Send
+ Sync
+ Default
+ core::ops::Add<Output = Self>
+ core::ops::Sub<Output = Self>
+ core::ops::Mul<Output = Self>
+ core::ops::Div<Output = Self>
+ core::ops::AddAssign
+ core::ops::SubAssign
+ core::ops::MulAssign
+ core::ops::DivAssign
+ core::cmp::PartialOrd

+ 'static
{
    const QUANTIZATION_LEVEL: QuantizationLevel;
    const QUANT_ZERO: Self;
}


/// Common base for all quantizable types (Alloc methods enabled)
#[cfg(feature = "alloc")]
pub trait QuantizedElementBase:
Copy
+ Clone
+ Send
+ Sync
+ Default
+ core::ops::Add<Output = Self>
+ core::ops::Sub<Output = Self>
+ core::ops::Mul<Output = Self>
+ core::ops::Div<Output = Self>
+ core::ops::AddAssign
+ core::ops::SubAssign
+ core::ops::MulAssign
+ core::ops::DivAssign
+ core::cmp::PartialOrd

+ core::fmt::Debug
+ core::fmt::Display
+ 'static
{
    const QUANTIZATION_LEVEL: QuantizationLevel;
    const QUANT_ZERO: Self;
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

#[cfg(feature = "support_64bit_indexing")]
impl QuantizedElementBase for u64 {
    const NUMBER_OF_BYTES: u8 = 8;
    const QUANTIZATION_LEVEL: QuantizationLevel = QuantizationLevel::Bit64;
    const QUANT_ZERO: Self = 0;
}

// Lol no we are not doing u128 or i128

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

#[cfg(feature = "support_64bit_indexing")]
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

#[cfg(feature = "support_64bit_values")]
impl QuantizedElementBase for f64 {
    const QUANTIZATION_LEVEL: QuantizationLevel = QuantizationLevel::Bit64;
    const QUANT_ZERO: Self = f64::ZERO;
}
