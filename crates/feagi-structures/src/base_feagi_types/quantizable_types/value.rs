use half::f16;
use crate::base_feagi_types::quantizable_types::{FeagiBaseQuantizationType, FeagiBaseSingleElementQuantizationType};
use crate::quantization_level::QuantizationLevel;

/// Defines a transparent value wrapper type and all `QuantizableValue` / operator / conversion impls.
#[macro_export]
macro_rules! define_quantizable_value_type_family {
    ($base_name:ident) => {
        #[repr(transparent)]
        #[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
        #[cfg_attr(feature = "alloc", derive(serde::Serialize, serde::Deserialize))]
        #[cfg_attr(feature = "alloc", serde(transparent))]
        pub struct $base_name<T: $crate::base_feagi_types::quantizable_types::QuantizableValueType>(pub T);

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableValueType> From<T> for $base_name<T> {
            #[inline(always)]
            fn from(value: T) -> Self {
                Self(value)
            }
        }

        #[cfg(feature = "alloc")]
        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableValueType> Default for $base_name<T> {
            #[inline(always)]
            fn default() -> Self {
                Self(T::default())
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableValueType> From<$base_name<T>> for f32 {
            #[inline(always)]
            fn from(value: $base_name<T>) -> Self {
                value.0.to_f32()
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableValueType> core::ops::Add for $base_name<T> {
            type Output = Self;
            #[inline(always)]
            fn add(self, rhs: Self) -> Self::Output {
                Self(self.0 + rhs.0)
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableValueType> core::ops::Sub for $base_name<T> {
            type Output = Self;
            #[inline(always)]
            fn sub(self, rhs: Self) -> Self::Output {
                Self(self.0 - rhs.0)
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableValueType> core::ops::Mul for $base_name<T> {
            type Output = Self;
            #[inline(always)]
            fn mul(self, rhs: Self) -> Self::Output {
                Self(self.0 * rhs.0)
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableValueType> core::ops::Div for $base_name<T> {
            type Output = Self;
            #[inline(always)]
            fn div(self, rhs: Self) -> Self::Output {
                Self(self.0 / rhs.0)
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableValueType> core::ops::AddAssign for $base_name<T> {
            #[inline(always)]
            fn add_assign(&mut self, rhs: Self) {
                self.0 += rhs.0;
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableValueType> core::ops::SubAssign for $base_name<T> {
            #[inline(always)]
            fn sub_assign(&mut self, rhs: Self) {
                self.0 -= rhs.0;
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableValueType> core::ops::MulAssign for $base_name<T> {
            #[inline(always)]
            fn mul_assign(&mut self, rhs: Self) {
                self.0 *= rhs.0;
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableValueType> core::ops::DivAssign for $base_name<T> {
            #[inline(always)]
            fn div_assign(&mut self, rhs: Self) {
                self.0 /= rhs.0;
            }
        }

        #[cfg(feature = "alloc")]
        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableValueType + core::fmt::Display> core::fmt::Display
            for $base_name<T>
        {
            #[inline(always)]
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableValueType> $crate::base_feagi_types::quantizable_types::FeagiBaseQuantizationType for $base_name<T> {
            const NUMBER_OF_BYTES: usize = T::NUMBER_OF_BYTES;

            #[inline(always)]
            fn saturating_add(self, other: Self) -> Self {
                Self(self.0.saturating_add(other.0))
            }

            #[inline(always)]
            fn checked_add(self, other: Self) -> Option<Self> {
                self.0.checked_add(other.0).map(Self)
            }

            #[inline(always)]
            fn saturating_sub(self, other: Self) -> Self {
                Self(self.0.saturating_sub(other.0))
            }

            #[inline(always)]
            fn checked_sub(self, other: Self) -> Option<Self> {
                self.0.checked_sub(other.0).map(Self)
            }

            #[inline(always)]
            fn saturating_mul(self, other: Self) -> Self {
                Self(self.0.saturating_mul(other.0))
            }

            #[inline(always)]
            fn checked_mul(self, other: Self) -> Option<Self> {
                self.0.checked_mul(other.0).map(Self)
            }

            #[inline(always)]
            fn checked_div(self, other: Self) -> Option<Self> {
                self.0.checked_div(other.0).map(Self)
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableValueType> $crate::base_feagi_types::quantizable_types::FeagiBaseSingleElementQuantizationType
            for $base_name<T>
        {
            const ZERO: Self = Self(T::ZERO);
            const ONE: Self = Self(T::ONE);
            const MAX_VALUE: Self = Self(T::MAX_VALUE);
            const MIN_VALUE: Self = Self(T::MIN_VALUE);
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableValueType> $crate::base_feagi_types::quantizable_types::QuantizableValueType for $base_name<T> {
            #[inline(always)]
            fn to_f32(self) -> f32 {
                self.0.to_f32()
            }

            #[inline(always)]
            fn from_f32(value: f32) -> Self {
                Self(T::from_f32(value))
            }
        }
    };
}


pub trait QuantizableValueType: FeagiBaseSingleElementQuantizationType + core::convert::Into<f32> {
    fn to_f32(self) -> f32;
    fn from_f32(value: f32) -> Self;
}


impl QuantizableValueType for u8 {
    #[inline(always)]
    fn to_f32(self) -> f32 {
        self as f32
    }

    #[inline(always)]
    fn from_f32(value: f32) -> Self {
        if value.is_nan() {
            0
        } else if value.is_sign_negative() {
            0
        } else if value > u8::MAX as f32 {
            u8::MAX
        } else {
            value as u8
        }
    }
}

impl QuantizableValueType for f16 {
    #[inline(always)]
    fn to_f32(self) -> f32 {
        self.to_f32()
    }

    #[inline(always)]
    fn from_f32(value: f32) -> Self {
        if value.is_nan() {
            f16::from_f32(0.0)
        } else if value > f16::MAX.to_f32() {
            f16::MAX
        } else if value < f16::MIN.to_f32() {
            f16::MIN
        } else {
            f16::from_f32(value)
        }
    }
}

impl QuantizableValueType for f32 {
    #[inline(always)]
    fn to_f32(self) -> f32 {
        self
    }

    #[inline(always)]
    fn from_f32(value: f32) -> Self {
        if value.is_infinite() {
            if value.is_sign_negative() { f32::MIN } else { f32::MAX }
        } else if value.is_nan() {
            0.0
        } else {
            value
        }
    }
}

// TODO f64

