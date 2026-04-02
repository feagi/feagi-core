use core::mem::size_of;
use half::f16;

/// Defines a transparent percent wrapper type and `QuantizablePercentType` / operator / conversion impls.
#[macro_export]
macro_rules! define_quantizable_percentage_type_family {
    ($base_name:ident) => {
        #[repr(transparent)]
        #[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
        #[cfg_attr(feature = "alloc", derive(serde::Serialize, serde::Deserialize))]
        #[cfg_attr(feature = "alloc", serde(transparent))]
        pub struct $base_name<T: $crate::base_quantizable::QuantizablePercentType>(pub T);

        impl<T: $crate::base_quantizable::QuantizablePercentType> From<T> for $base_name<T> {
            #[inline(always)]
            fn from(value: T) -> Self {
                Self(value)
            }
        }

        #[cfg(feature = "alloc")]
        impl<T: $crate::base_quantizable::QuantizablePercentType> Default for $base_name<T> {
            #[inline(always)]
            fn default() -> Self {
                Self(T::default())
            }
        }

        impl<T: $crate::base_quantizable::QuantizablePercentType> From<$base_name<T>> for f32 {
            #[inline(always)]
            fn from(value: $base_name<T>) -> Self {
                value.0.to_f32()
            }
        }

        impl<T: $crate::base_quantizable::QuantizablePercentType> core::ops::Add for $base_name<T> {
            type Output = Self;
            #[inline(always)]
            fn add(self, rhs: Self) -> Self::Output {
                Self(self.0 + rhs.0)
            }
        }

        impl<T: $crate::base_quantizable::QuantizablePercentType> core::ops::Sub for $base_name<T> {
            type Output = Self;
            #[inline(always)]
            fn sub(self, rhs: Self) -> Self::Output {
                Self(self.0 - rhs.0)
            }
        }

        impl<T: $crate::base_quantizable::QuantizablePercentType> core::ops::Mul for $base_name<T> {
            type Output = Self;
            #[inline(always)]
            fn mul(self, rhs: Self) -> Self::Output {
                Self(self.0 * rhs.0)
            }
        }

        impl<T: $crate::base_quantizable::QuantizablePercentType> core::ops::Div for $base_name<T> {
            type Output = Self;
            #[inline(always)]
            fn div(self, rhs: Self) -> Self::Output {
                Self(self.0 / rhs.0)
            }
        }

        impl<T: $crate::base_quantizable::QuantizablePercentType> core::ops::AddAssign for $base_name<T> {
            #[inline(always)]
            fn add_assign(&mut self, rhs: Self) {
                self.0 += rhs.0;
            }
        }

        impl<T: $crate::base_quantizable::QuantizablePercentType> core::ops::SubAssign for $base_name<T> {
            #[inline(always)]
            fn sub_assign(&mut self, rhs: Self) {
                self.0 -= rhs.0;
            }
        }

        impl<T: $crate::base_quantizable::QuantizablePercentType> core::ops::MulAssign for $base_name<T> {
            #[inline(always)]
            fn mul_assign(&mut self, rhs: Self) {
                self.0 *= rhs.0;
            }
        }

        impl<T: $crate::base_quantizable::QuantizablePercentType> core::ops::DivAssign for $base_name<T> {
            #[inline(always)]
            fn div_assign(&mut self, rhs: Self) {
                self.0 /= rhs.0;
            }
        }

        #[cfg(feature = "alloc")]
        impl<T: $crate::base_quantizable::QuantizablePercentType> core::fmt::Display for $base_name<T> {
            #[inline(always)]
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl<T: $crate::base_quantizable::QuantizablePercentType> $crate::base_quantizable::QuantizablePercentType
            for $base_name<T>
        {
            const NUMBER_OF_BYTES: usize = T::NUMBER_OF_BYTES;
            const ZERO_PERCENT: Self = Self(T::ZERO_PERCENT);
            const HUNDRED_PERCENT: Self = Self(T::HUNDRED_PERCENT);

            #[inline(always)]
            fn clamped_add(self, other: Self) -> Self {
                Self(self.0.clamped_add(other.0))
            }

            #[inline(always)]
            fn clamped_sub(self, other: Self) -> Self {
                Self(self.0.clamped_sub(other.0))
            }

            #[inline(always)]
            fn clamped_mul(self, other: Self) -> Self {
                Self(self.0.clamped_mul(other.0))
            }

            #[inline(always)]
            fn clamped_div(self, other: Self) -> Self {
                Self(self.0.clamped_div(other.0))
            }

            #[inline(always)]
            fn to_f32(self) -> f32 {
                self.0.to_f32()
            }

            #[inline(always)]
            fn from_f32_clamped(value: f32) -> Self {
                Self(T::from_f32_clamped(value))
            }
        }
    };
}


#[cfg(not(feature = "alloc"))]
pub trait QuantizablePercentType:
    Copy
    + core::clone::Clone
    + Send
    + Sync
    + core::convert::Into<f32>
    + core::cmp::PartialOrd
    + core::ops::Add<Output = Self>
    + core::ops::Sub<Output = Self>
    + core::ops::Mul<Output = Self>
    + core::ops::Div<Output = Self>
    + core::ops::AddAssign
    + core::ops::SubAssign
    + core::ops::MulAssign
    + core::ops::DivAssign
    + 'static
{
    const NUMBER_OF_BYTES: usize;
    const ZERO_PERCENT: Self;
    const HUNDRED_PERCENT: Self;
    fn clamped_add(self, other: Self) -> Self;
    fn clamped_sub(self, other: Self) -> Self;
    fn clamped_mul(self, other: Self) -> Self;
    fn clamped_div(self, other: Self) -> Self;
    fn to_f32(self) -> f32;
    fn from_f32_clamped(value: f32) -> Self;
}

#[cfg(feature = "alloc")]
pub trait QuantizablePercentType:
    Copy
    + core::clone::Clone
    + Send
    + Sync
    + core::fmt::Debug
    + core::fmt::Display
    + core::default::Default
    + core::convert::Into<f32>
    + core::cmp::PartialOrd
    + core::ops::Add<Output = Self>
    + core::ops::Sub<Output = Self>
    + core::ops::Mul<Output = Self>
    + core::ops::Div<Output = Self>
    + core::ops::AddAssign
    + core::ops::SubAssign
    + core::ops::MulAssign
    + core::ops::DivAssign
    + 'static
{
    const NUMBER_OF_BYTES: usize;
    const ZERO_PERCENT: Self;
    const HUNDRED_PERCENT: Self;
    fn clamped_add(self, other: Self) -> Self;
    fn clamped_sub(self, other: Self) -> Self;
    fn clamped_mul(self, other: Self) -> Self;
    fn clamped_div(self, other: Self) -> Self;
    fn to_f32(self) -> f32;
    fn from_f32_clamped(value: f32) -> Self;
}

impl QuantizablePercentType for f32 {
    const NUMBER_OF_BYTES: usize = size_of::<Self>();
    const ZERO_PERCENT: Self = 0.0;
    const HUNDRED_PERCENT: Self = 1.0;

    #[inline(always)]
    fn clamped_add(self, other: Self) -> Self {
        let value = self + other;

        if value.is_infinite() {
            if value.is_sign_negative() { Self::ZERO_PERCENT } else { Self::HUNDRED_PERCENT }
        } else if value.is_nan() {
            Self::ZERO_PERCENT
        } else {
            value.clamp(Self::ZERO_PERCENT, Self::HUNDRED_PERCENT)
        }
    }

    #[inline(always)]
    fn clamped_sub(self, other: Self) -> Self {
        let value = self - other;

        if value.is_infinite() {
            if value.is_sign_negative() { Self::ZERO_PERCENT } else { Self::HUNDRED_PERCENT }
        } else if value.is_nan() {
            Self::ZERO_PERCENT
        } else {
            value.clamp(Self::ZERO_PERCENT, Self::HUNDRED_PERCENT)
        }
    }

    #[inline(always)]
    fn clamped_mul(self, other: Self) -> Self {
        let value = self * other;
        if value.is_infinite() {
            if value.is_sign_negative() { Self::ZERO_PERCENT } else { Self::HUNDRED_PERCENT }
        } else if value.is_nan() {
            Self::ZERO_PERCENT
        } else {
            value.clamp(Self::ZERO_PERCENT, Self::HUNDRED_PERCENT)
        }
    }

    #[inline(always)]
    fn clamped_div(self, other: Self) -> Self {
        let value = self / other;
        if value.is_infinite() {
            if value.is_sign_negative() { Self::ZERO_PERCENT } else { Self::HUNDRED_PERCENT }
        } else if value.is_nan() {
            Self::ZERO_PERCENT
        } else {
            value.clamp(Self::ZERO_PERCENT, Self::HUNDRED_PERCENT)
        }
    }

    #[inline(always)]
    fn to_f32(self) -> f32 {
        self
    }

    #[inline(always)]
    fn from_f32_clamped(value: f32) -> Self {
        if value.is_infinite() {
            if value.is_sign_negative() { Self::ZERO_PERCENT } else { Self::HUNDRED_PERCENT }
        } else if value.is_nan() {
            Self::ZERO_PERCENT
        } else {
            value.clamp(Self::ZERO_PERCENT, Self::HUNDRED_PERCENT)
        }
    }
}

impl QuantizablePercentType for f16 {
    const NUMBER_OF_BYTES: usize = size_of::<f16>();
    const ZERO_PERCENT: Self = f16::ZERO;
    const HUNDRED_PERCENT: Self = f16::ONE;

    #[inline(always)]
    fn clamped_add(self, other: Self) -> Self {
        let value = self + other;

        if value.is_infinite() {
            if value.is_sign_negative() { Self::ZERO_PERCENT } else { Self::HUNDRED_PERCENT }
        } else if value.is_nan() {
            Self::ZERO_PERCENT
        } else {
            value.clamp(Self::ZERO_PERCENT, Self::HUNDRED_PERCENT)
        }
    }

    #[inline(always)]
    fn clamped_sub(self, other: Self) -> Self {
        let value = self - other;

        if value.is_infinite() {
            if value.is_sign_negative() { Self::ZERO_PERCENT } else { Self::HUNDRED_PERCENT }
        } else if value.is_nan() {
            Self::ZERO_PERCENT
        } else {
            value.clamp(Self::ZERO_PERCENT, Self::HUNDRED_PERCENT)
        }
    }

    #[inline(always)]
    fn clamped_mul(self, other: Self) -> Self {
        let value = self * other;
        if value.is_infinite() {
            if value.is_sign_negative() { Self::ZERO_PERCENT } else { Self::HUNDRED_PERCENT }
        } else if value.is_nan() {
            Self::ZERO_PERCENT
        } else {
            value.clamp(Self::ZERO_PERCENT, Self::HUNDRED_PERCENT)
        }
    }

    #[inline(always)]
    fn clamped_div(self, other: Self) -> Self {
        let value = self / other;
        if value.is_infinite() {
            if value.is_sign_negative() { Self::ZERO_PERCENT } else { Self::HUNDRED_PERCENT }
        } else if value.is_nan() {
            Self::ZERO_PERCENT
        } else {
            value.clamp(Self::ZERO_PERCENT, Self::HUNDRED_PERCENT)
        }
    }

    #[inline(always)]
    fn to_f32(self) -> f32 {
        core::convert::Into::<f32>::into(self)
    }

    #[inline(always)]
    fn from_f32_clamped(value: f32) -> Self {
        if value.is_infinite() {
            if value.is_sign_negative() {
                Self::ZERO_PERCENT
            } else {
                Self::HUNDRED_PERCENT
            }
        } else if value.is_nan() {
            Self::ZERO_PERCENT
        } else {
            f16::from_f32(value.clamp(0.0, 1.0))
        }
    }
}

/// `u8` percentages use **0 = 0%, 255 = 100%** (linear map to `[0.0, 1.0]`).
impl QuantizablePercentType for u8 {
    const NUMBER_OF_BYTES: usize = size_of::<u8>();
    const ZERO_PERCENT: Self = 0;
    const HUNDRED_PERCENT: Self = 255;

    #[inline(always)]
    fn clamped_add(self, other: Self) -> Self {
        Self::from_f32_clamped(self.to_f32() + other.to_f32())
    }

    #[inline(always)]
    fn clamped_sub(self, other: Self) -> Self {
        Self::from_f32_clamped(self.to_f32() - other.to_f32())
    }

    #[inline(always)]
    fn clamped_mul(self, other: Self) -> Self {
        Self::from_f32_clamped(self.to_f32() * other.to_f32())
    }

    #[inline(always)]
    fn clamped_div(self, other: Self) -> Self {
        Self::from_f32_clamped(self.to_f32() / other.to_f32())
    }

    #[inline(always)]
    fn to_f32(self) -> f32 {
        self as f32 / Self::HUNDRED_PERCENT as f32
    }

    #[inline(always)]
    fn from_f32_clamped(value: f32) -> Self {
        if value.is_infinite() {
            if value.is_sign_negative() {
                Self::ZERO_PERCENT
            } else {
                Self::HUNDRED_PERCENT
            }
        } else if value.is_nan() {
            Self::ZERO_PERCENT
        } else {
            let v = value.clamp(0.0, 1.0);
            (v * Self::HUNDRED_PERCENT as f32).round().max(0.0).min(255.0) as u8
        }
    }
}
