//! Values that hold some sort of decimal (float) value

// TODO Equal check with epsilon

use crate::values::percentage::PercentageUnsigned;
use crate::values::quantizable::custom_data_types::StorageF8;
use crate::values::quantizable::QuantizedElementBase;
use half::f16;

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
    // TODO to and from other quant levels!
    fn to_storage_f8(self) -> StorageF8;
    fn from_storage_f8(v: StorageF8) -> Self;

    fn to_f16(self) -> f16;
    fn from_f16(v: f16) -> Self;

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
