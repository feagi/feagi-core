//! Values that hold some sort of decimal (float) value. Note that yes, different hardwares support
//! different values types

// TODO Equal check with epsilon

use crate::values::percentage::PercentageUnsigned;
use crate::values::quantizable::custom_data_types::StorageF8;
use crate::values::quantizable::quantization_level_packing::QuantizationLevelPacking;
use crate::values::quantizable::QuantizedElementBase;
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
    + Sized
    + 'static
    + QuantizedElementBase
{
    fn to_storage_f8(self) -> StorageF8;
    fn from_storage_f8(v: StorageF8) -> Self;

    fn to_f16(self) -> f16;
    fn from_f16(v: f16) -> Self;

    fn to_bf16(self) -> bf16;

    fn from_bf16(v: bf16) -> Self;

    fn to_f32(self) -> f32;
    fn from_f32(value: f32) -> Self;

    fn to_f64(self) -> f64;
    fn from_f64(value: f64) -> Self;

    fn from_unsigned_percentage(v: PercentageUnsigned) -> Self;
}

impl QuantizedDecimalTrait for StorageF8 {
    fn to_storage_f8(self) -> StorageF8 {
        todo!()
    }

    fn from_storage_f8(v: StorageF8) -> Self {
        todo!()
    }

    fn to_f16(self) -> f16 {
        todo!()
    }

    fn from_f16(v: f16) -> Self {
        todo!()
    }

    fn to_bf16(self) -> bf16 {
        todo!()
    }

    fn from_bf16(v: bf16) -> Self {
        todo!()
    }

    fn to_f32(self) -> f32 {
        StorageF8::to_f32(self)
    }

    fn from_f32(value: f32) -> Self {
        StorageF8::from_f32(value)
    }

    fn to_f64(self) -> f64 {
        todo!()
    }

    fn from_f64(value: f64) -> Self {
        todo!()
    }

    fn from_unsigned_percentage(v: PercentageUnsigned) -> Self {
        todo!()
    }
}

impl QuantizedDecimalTrait for f16 {
    fn to_storage_f8(self) -> StorageF8 {
        todo!()
    }

    fn from_storage_f8(v: StorageF8) -> Self {
        todo!()
    }

    fn to_f16(self) -> f16 {
        todo!()
    }

    fn from_f16(v: f16) -> Self {
        todo!()
    }

    fn to_bf16(self) -> bf16 {
        todo!()
    }

    fn from_bf16(v: bf16) -> Self {
        todo!()
    }

    fn to_f32(self) -> f32 {
        f16::to_f32(self)
    }

    fn from_f32(value: f32) -> Self {
        f16::from_f32(value)
    }

    fn to_f64(self) -> f64 {
        todo!()
    }

    fn from_f64(value: f64) -> Self {
        todo!()
    }

    fn from_unsigned_percentage(v: PercentageUnsigned) -> Self {
        todo!()
    }
}

impl QuantizedDecimalTrait for bf16 {
    fn to_storage_f8(self) -> StorageF8 {
        todo!()
    }

    fn from_storage_f8(v: StorageF8) -> Self {
        todo!()
    }

    fn to_f16(self) -> f16 {
        todo!()
    }

    fn from_f16(v: f16) -> Self {
        todo!()
    }

    fn to_bf16(self) -> bf16 {
        todo!()
    }

    fn from_bf16(v: bf16) -> Self {
        todo!()
    }

    fn to_f32(self) -> f32 {
        bf16::to_f32(self)
    }

    fn from_f32(value: f32) -> Self {
        bf16::from_f32(value)
    }

    fn to_f64(self) -> f64 {
        todo!()
    }

    fn from_f64(value: f64) -> Self {
        todo!()
    }

    fn from_unsigned_percentage(v: PercentageUnsigned) -> Self {
        todo!()
    }
}

impl QuantizedDecimalTrait for f32 {
    fn to_storage_f8(self) -> StorageF8 {
        todo!()
    }

    fn from_storage_f8(v: StorageF8) -> Self {
        todo!()
    }

    fn to_f16(self) -> f16 {
        todo!()
    }

    fn from_f16(v: f16) -> Self {
        todo!()
    }

    fn to_bf16(self) -> bf16 {
        todo!()
    }

    fn from_bf16(v: bf16) -> Self {
        todo!()
    }

    fn to_f32(self) -> f32 {
        self
    }

    fn from_f32(value: f32) -> Self {
        value
    }

    fn to_f64(self) -> f64 {
        self as f64
    }

    fn from_f64(value: f64) -> Self {
        value.to_f32()
    }

    fn from_unsigned_percentage(v: PercentageUnsigned) -> Self {
        v.to_f32_0_1()
    }
}

impl QuantizedDecimalTrait for f64 {
    fn to_storage_f8(self) -> StorageF8 {
        todo!()
    }

    fn from_storage_f8(v: StorageF8) -> Self {
        todo!()
    }

    fn to_f16(self) -> f16 {
        todo!()
    }

    fn from_f16(v: f16) -> Self {
        todo!()
    }

    fn to_bf16(self) -> bf16 {
        todo!()
    }

    fn from_bf16(v: bf16) -> Self {
        todo!()
    }

    fn to_f32(self) -> f32 {
        self as f32
    }

    fn from_f32(value: f32) -> Self {
        value as f64
    }

    fn to_f64(self) -> f64 {
        todo!()
    }

    fn from_f64(value: f64) -> Self {
        todo!()
    }

    fn from_unsigned_percentage(v: PercentageUnsigned) -> Self {
        todo!()
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
            pub fn to_f32(self) -> f32 {
                self.0.to_f32()
            }

            pub fn from_f32(value: f32) -> Self {
                Self(Q::from_f32(value))
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
