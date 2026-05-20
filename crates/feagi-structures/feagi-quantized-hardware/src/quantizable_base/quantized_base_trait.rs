use half::f16;
use crate::quantization_shared::QuantizationLevel;
use crate::storage_f8::StorageF8;

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
    const NUMBER_OF_BYTES: u8;
    const QUANTIZATION_LEVEL: QuantizationLevel;
}


/// Common base for all quantizable types (Alloc methods enabled)
#[cfg(feature = "alloc")]
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

+ core::fmt::Debug
+ core::fmt::Display
+ 'static
{
    const NUMBER_OF_BYTES: u8;
    const QUANTIZATION_LEVEL: QuantizationLevel;
}

impl QuantizedBaseTrait for u8 {
    const NUMBER_OF_BYTES: u8 = 1;
    const QUANTIZATION_LEVEL: QuantizationLevel = QuantizationLevel::Bit8;
}

impl QuantizedBaseTrait for u16 {
    const NUMBER_OF_BYTES: u8 = 2;
    const QUANTIZATION_LEVEL: QuantizationLevel = QuantizationLevel::Bit16;
}

impl QuantizedBaseTrait for u32 {
    const NUMBER_OF_BYTES: u8 = 4;
    const QUANTIZATION_LEVEL: QuantizationLevel = QuantizationLevel::Bit32;
}

#[cfg(feature = "support_64bit_indexing")]
impl QuantizedBaseTrait for u64 {
    const NUMBER_OF_BYTES: u8 = 8;
    const QUANTIZATION_LEVEL: QuantizationLevel = QuantizationLevel::Bit64;
}

// Lol no we are not doing u128 or i128

impl QuantizedBaseTrait for i8 {
    const NUMBER_OF_BYTES: u8 = 1;
    const QUANTIZATION_LEVEL: QuantizationLevel = QuantizationLevel::Bit8;
}

impl QuantizedBaseTrait for i16 {
    const NUMBER_OF_BYTES: u8 = 2;
    const QUANTIZATION_LEVEL: QuantizationLevel = QuantizationLevel::Bit16;
}

impl QuantizedBaseTrait for i32 {
    const NUMBER_OF_BYTES: u8 = 4;
    const QUANTIZATION_LEVEL: QuantizationLevel = QuantizationLevel::Bit32;
}

#[cfg(feature = "support_64bit_indexing")]
impl QuantizedBaseTrait for i64 {
    const NUMBER_OF_BYTES: u8 = 8;
    const QUANTIZATION_LEVEL: QuantizationLevel = QuantizationLevel::Bit64;
}

// A bad choice for computation
impl QuantizedBaseTrait for StorageF8 {
    const NUMBER_OF_BYTES: u8 = 1;
    const QUANTIZATION_LEVEL: QuantizationLevel = QuantizationLevel::Bit8;
}

impl QuantizedBaseTrait for f16 {
    const NUMBER_OF_BYTES: u8 = 2;
    const QUANTIZATION_LEVEL: QuantizationLevel = QuantizationLevel::Bit16;
}

impl QuantizedBaseTrait for f32 {
    const NUMBER_OF_BYTES: u8 = 4;
    const QUANTIZATION_LEVEL: QuantizationLevel = QuantizationLevel::Bit32;
}

#[cfg(feature = "support_64bit_values")]
impl QuantizedBaseTrait for f64 {
    const NUMBER_OF_BYTES: u8 = 8;
    const QUANTIZATION_LEVEL: QuantizationLevel = QuantizationLevel::Bit64;
}

