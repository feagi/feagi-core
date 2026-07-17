//! Values that hold some sort of decimal (float) value. Note that yes, different hardwares support
//! different values types

// TODO Equal check with epsilon

use crate::values::quantizable::custom_data_types::StorageF8;
use crate::values::quantizable::quantization_level_packing::QuantizationLevelPacking;
use crate::values::quantizable::{PercentageUnsigned, QuantizedElementBase};
use half::{bf16, f16};

/// Represents a value that is represented as a decimal number, main backbone for computations
#[repr(u8)]
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum DecimalQuantizationLevel {
    F16 = 0,
    BF16 = 1,
    F32 = 2,
    F64 = 3,
    StorageF8 = 4,
    // We can support a max of 16 quants
}

impl Into<u8> for DecimalQuantizationLevel {
    fn into(self) -> u8 {
        self as u8
    }
}

impl TryFrom<u8> for DecimalQuantizationLevel {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(DecimalQuantizationLevel::F16),
            1 => Ok(DecimalQuantizationLevel::BF16),
            2 => Ok(DecimalQuantizationLevel::F32),
            3 => Ok(DecimalQuantizationLevel::F64),
            4 => Ok(DecimalQuantizationLevel::StorageF8),
            _ => Err(()),
        }
    }
}

impl QuantizationLevelPacking for DecimalQuantizationLevel {
    const NUMBER_BITS: usize = 4; // 16 quants

    unsafe fn from_packed_byte(byte: u8) -> Self {
        core::mem::transmute(byte)
    }
}

/// Quantizable data for some decimal value (float)
pub trait QuantizedDecimalTrait:
Copy
+ Clone
+ Send
+ Sync
+ Default
+ core::ops::Add<Output=Self>
+ core::ops::Sub<Output=Self>
+ core::ops::Mul<Output=Self>
+ core::ops::Div<Output=Self>
+ core::ops::AddAssign
+ core::ops::SubAssign
+ core::ops::MulAssign
+ core::ops::DivAssign
+ core::cmp::PartialOrd
+ core::fmt::Debug
+ core::fmt::Display
+ Sized
+ 'static
+ QuantizedElementBase
{
    const LEVEL: DecimalQuantizationLevel;

    fn quant_clamp(&self, min: Self, max: Self) -> Self;

    fn quant_to_storage_f8(self) -> StorageF8;

    fn quant_to_f16(self) -> f16;

    fn quant_to_bf16(self) -> bf16;

    fn quant_to_f32(self) -> f32;

    fn quant_to_f64(self) -> f64;

    /// Creates from a decimal of another quantization
    fn from_quantization<FromQuant: QuantizedDecimalTrait>(value: FromQuant) -> Self;

    /// Converts to a decimal of another quantization
    fn to_quantization<ToQuant: QuantizedDecimalTrait>(self) -> ToQuant {
        ToQuant::from_quantization(self)
    }

    fn scale_by_unsigned_percentage<OTHER: QuantizedDecimalTrait>(self, p: PercentageUnsigned<OTHER>) -> Self {
        // percentages will always be in valid range, we dont need a checked conversion
        self * Self::from_quantization::<OTHER>(p.get_decimal())
    }

    fn scale_by_same_quant_unsigned_percentage(self, p: &PercentageUnsigned<Self>) -> Self {
        self * p.get_decimal()
    }
}

impl QuantizedDecimalTrait for StorageF8 {
    const LEVEL: DecimalQuantizationLevel = DecimalQuantizationLevel::StorageF8;

    fn quant_clamp(&self, min: Self, max: Self) -> Self {
        todo!()
    }

    fn quant_to_storage_f8(self) -> StorageF8 {
        todo!()
    }

    fn quant_to_f16(self) -> f16 {
        todo!()
    }

    fn quant_to_bf16(self) -> bf16 {
        todo!()
    }

    fn quant_to_f32(self) -> f32 {
        todo!()
    }

    fn quant_to_f64(self) -> f64 {
        todo!()
    }

    fn from_quantization<FromQuant: QuantizedDecimalTrait>(value: FromQuant) -> Self {
        todo!()
    }
}

impl QuantizedDecimalTrait for f16 {
    const LEVEL: DecimalQuantizationLevel = DecimalQuantizationLevel::F16;

    fn quant_clamp(&self, min: Self, max: Self) -> Self {
        self.clamp(min, max)
    }

    fn quant_to_storage_f8(self) -> StorageF8 {
        todo!()
    }

    fn quant_to_f16(self) -> f16 {
        self
    }

    fn quant_to_bf16(self) -> bf16 {
        bf16::from_f32(self.to_f32())
    }

    fn quant_to_f32(self) -> f32 {
        f16::to_f32(self)
    }

    fn quant_to_f64(self) -> f64 {
        f16::to_f64(self)
    }

    fn from_quantization<FromQuant: QuantizedDecimalTrait>(value: FromQuant) -> Self {
        value.quant_to_f16()
    }
}

impl QuantizedDecimalTrait for bf16 {
    const LEVEL: DecimalQuantizationLevel = DecimalQuantizationLevel::BF16;

    fn quant_clamp(&self, min: Self, max: Self) -> Self {
        self.clamp(min, max)
    }

    fn quant_to_storage_f8(self) -> StorageF8 {
        todo!()
    }

    fn quant_to_f16(self) -> f16 {
        f16::from_f32(self.to_f32())
    }

    fn quant_to_bf16(self) -> bf16 {
        self
    }

    fn quant_to_f32(self) -> f32 {
        bf16::to_f32(self)
    }

    fn quant_to_f64(self) -> f64 {
        bf16::to_f64(self)
    }

    fn from_quantization<FromQuant: QuantizedDecimalTrait>(value: FromQuant) -> Self {
        value.quant_to_bf16()
    }
}

impl QuantizedDecimalTrait for f32 {
    const LEVEL: DecimalQuantizationLevel = DecimalQuantizationLevel::F32;

    fn quant_clamp(&self, min: Self, max: Self) -> Self {
        self.clamp(min, max)
    }

    fn quant_to_storage_f8(self) -> StorageF8 {
        todo!()
    }

    fn quant_to_f16(self) -> f16 {
        f16::from_f32(self)
    }

    fn quant_to_bf16(self) -> bf16 {
        bf16::from_f32(self)
    }

    fn quant_to_f32(self) -> f32 {
        self
    }

    fn quant_to_f64(self) -> f64 {
        self as f64
    }

    fn from_quantization<FromQuant: QuantizedDecimalTrait>(value: FromQuant) -> Self {
        value.quant_to_f32()
    }
}

impl QuantizedDecimalTrait for f64 {
    const LEVEL: DecimalQuantizationLevel = DecimalQuantizationLevel::F64;

    fn quant_clamp(&self, min: Self, max: Self) -> Self {
        self.clamp(min, max)
    }

    fn quant_to_storage_f8(self) -> StorageF8 {
        todo!()
    }

    fn quant_to_f16(self) -> f16 {
        f16::from_f64(self)
    }

    fn quant_to_bf16(self) -> bf16 {
        bf16::from_f64(self)
    }

    fn quant_to_f32(self) -> f32 {
        self as f32
    }

    fn quant_to_f64(self) -> f64 {
        self
    }

    fn from_quantization<FromQuant: QuantizedDecimalTrait>(value: FromQuant) -> Self {
        value.quant_to_f64()
    }
}

/// Creates a wrapper for quantized decimal values
#[macro_export]
macro_rules! create_wrapped_quantized_decimal {
    (
        $(#[$meta:meta])*
        $vis:vis $struct_name:ident
    ) => {
        $(#[$meta])*
        #[repr(transparent)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        $vis struct $struct_name<Q: $crate::values::quantizable::QuantizedDecimalTrait>(Q);

        impl<Q: $crate::values::quantizable::QuantizedDecimalTrait> $struct_name<Q> {
            pub const LEVEL: $crate::values::quantizable::DecimalQuantizationLevel = Q::LEVEL;
            pub const QUANT_ZERO: Self = Self::const_new(Q::QUANT_ZERO);
            pub const QUANT_ONE: Self = Self::const_new(Q::QUANT_ONE);

            pub const fn const_new(value: Q) -> Self
            {
                Self(value)
            }

            pub const fn const_deref(self) -> Q
            {
                self.0
            }

            pub fn new(v: Q) -> Self {
                Self(v)
            }

            pub fn quant_to_storage_f8(self) -> $crate::values::quantizable::custom_data_types::StorageF8 {
                self.0.quant_to_storage_f8()
            }

            pub fn quant_to_f16(self) -> half::f16 {
                self.0.quant_to_f16()
            }

            pub fn quant_to_bf16(self) -> half::bf16 {
                self.0.quant_to_bf16()
            }

            pub fn quant_to_f32(self) -> f32 {
                self.0.quant_to_f32()
            }

            pub fn quant_to_f64(self) -> f64 {
                self.0.quant_to_f64()
            }

            /// Creates from a decimal of another quantization
            pub fn from_quantization<FromQuant: $crate::values::quantizable::QuantizedDecimalTrait>(value: FromQuant) -> Self {
                Self(Q::from_quantization(value))
            }

            /// Converts to a decimal of another quantization
            pub fn to_quantization<ToQuant: $crate::values::quantizable::QuantizedDecimalTrait>(self) -> ToQuant {
                self.0.to_quantization()
            }

            pub fn scale_by_unsigned_percentage<OTHER: $crate::values::quantizable::QuantizedDecimalTrait>(self, p: $crate::values::quantizable::PercentageUnsigned<OTHER>) -> Self {
                Self(self.0.scale_by_unsigned_percentage(p))
            }

            pub fn scale_by_same_quant_unsigned_percentage(self, p: &$crate::values::quantizable::PercentageUnsigned<Q>) -> Self {
                Self(self.0.scale_by_same_quant_unsigned_percentage(p))
            }

            /// Extracts the inner quantized decimal
            pub fn deref(self) -> Q {
                self.0
            }
        }

        // NOTE: Into<Q> for $struct_name<Q> is not needed!

        impl<Q: $crate::values::quantizable::QuantizedDecimalTrait> From<Q> for $struct_name<Q> {
            fn from(value: Q) -> Self {
                Self(value)
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedDecimalTrait> From<&Q> for &$struct_name<Q> {
            fn from(value: &Q) -> Self {
                // tRust me bro
                unsafe { &*(value as *const Q as *const Self) }
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedDecimalTrait> AsRef<Q> for $struct_name<Q> {
            fn as_ref(&self) -> &Q {
                &self.0
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedDecimalTrait> AsMut<Q> for $struct_name<Q> {
            fn as_mut(&mut self) -> &mut Q {
                &mut self.0
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedDecimalTrait> core::ops::Add for $struct_name<Q> {
            type Output = Self;
            fn add(self, rhs: Self) -> Self::Output {
                Self(self.0 + rhs.0)
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedDecimalTrait> core::ops::Sub for $struct_name<Q> {
            type Output = Self;
            fn sub(self, rhs: Self) -> Self::Output {
                Self(self.0 - rhs.0)
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedDecimalTrait> core::ops::Mul for $struct_name<Q> {
            type Output = Self;
            fn mul(self, rhs: Self) -> Self::Output {
                Self(self.0 * rhs.0)
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedDecimalTrait> core::ops::Div for $struct_name<Q> {
            type Output = Self;
            fn div(self, rhs: Self) -> Self::Output {
                Self(self.0 / rhs.0)
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedDecimalTrait> core::ops::AddAssign for $struct_name<Q> {
            fn add_assign(&mut self, rhs: Self) {
                self.0 += rhs.0;
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedDecimalTrait> core::ops::SubAssign for $struct_name<Q> {
            fn sub_assign(&mut self, rhs: Self) {
                self.0 -= rhs.0;
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedDecimalTrait> core::ops::MulAssign for $struct_name<Q> {
            fn mul_assign(&mut self, rhs: Self) {
                self.0 *= rhs.0;
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedDecimalTrait> core::ops::DivAssign for $struct_name<Q> {
            fn div_assign(&mut self, rhs: Self) {
                self.0 /= rhs.0;
            }
        }
    };
}
