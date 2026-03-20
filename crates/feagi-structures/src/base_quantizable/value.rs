use core::fmt::{Debug, Display};
use half::f16;

#[cfg(feature = "support_64bit_indexing_quantization")]
pub type QuantizableValueF64 = f64;
pub type QuantizableValueF32 = f32;
pub type QuantizableValueF16 = f16;
pub type QuantizableValueU8 = u8;

#[cfg(not(feature = "alloc"))]
pub trait QuantizableValue:
    Copy
    + Clone
    + Send
    + Sync
    + Into<f32>
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
    const ZERO: Self;
    const ONE: Self;
    const MAX_VALUE: Self;
    const MIN_VALUE: Self;
    fn saturating_add(self, other: Self) -> Self;
    fn checked_add(self, other: Self) -> Option<Self>;
    fn saturating_sub(self, other: Self) -> Self;
    fn checked_sub(self, other: Self) -> Option<Self>;
    fn saturating_mul(self, other: Self) -> Self;
    fn checked_mul(self, other: Self) -> Option<Self>;
    fn checked_div(self, other: Self) -> Option<Self>;
    fn to_f32(self) -> f32;
    fn from_f32(value: f32) -> Self;
}

#[cfg(feature = "alloc")]
pub trait QuantizableValue:
    Copy
    + Clone
    + Send
    + Sync
    + Debug
    + Display
    + Default
    + Into<f32>
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
    const ZERO: Self;
    const ONE: Self;
    const MAX_VALUE: Self;
    const MIN_VALUE: Self;
    fn saturating_add(self, other: Self) -> Self;
    fn checked_add(self, other: Self) -> Option<Self>;
    fn saturating_sub(self, other: Self) -> Self;
    fn checked_sub(self, other: Self) -> Option<Self>;
    fn saturating_mul(self, other: Self) -> Self;
    fn checked_mul(self, other: Self) -> Option<Self>;
    fn checked_div(self, other: Self) -> Option<Self>;
    fn to_f32(self) -> f32;
    fn from_f32(value: f32) -> Self;
}

#[cfg(feature = "support_64bit_indexing_quantization")]
impl QuantizableValue for f64 {
    const NUMBER_OF_BYTES: usize = size_of::<f64>();
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;
    const MAX_VALUE: Self = f64::MAX;
    const MIN_VALUE: Self = f64::MIN;

    #[inline(always)]
    fn saturating_add(self, other: Self) -> Self {
        let value = self + other;
        if value.is_infinite() {
            if value.is_sign_negative() { f64::MIN } else { f64::MAX }
        } else if value.is_nan() {
            0.0
        } else {
            value
        }
    }

    #[inline(always)]
    fn checked_add(self, other: Self) -> Option<Self> {
        let value = self + other;
        if value.is_finite() { Some(value) } else { None }
    }

    #[inline(always)]
    fn saturating_sub(self, other: Self) -> Self {
        let value = self - other;
        if value.is_infinite() {
            if value.is_sign_negative() { f64::MIN } else { f64::MAX }
        } else if value.is_nan() {
            0.0
        } else {
            value
        }
    }

    #[inline(always)]
    fn checked_sub(self, other: Self) -> Option<Self> {
        let value = self - other;
        if value.is_finite() { Some(value) } else { None }
    }

    #[inline(always)]
    fn saturating_mul(self, other: Self) -> Self {
        let value = self * other;
        if value.is_infinite() {
            if value.is_sign_negative() { f64::MIN } else { f64::MAX }
        } else if value.is_nan() {
            0.0
        } else {
            value
        }
    }

    #[inline(always)]
    fn checked_mul(self, other: Self) -> Option<Self> {
        let value = self * other;
        if value.is_finite() { Some(value) } else { None }
    }

    #[inline(always)]
    fn checked_div(self, other: Self) -> Option<Self> {
        let value = self / other;
        if value.is_finite() { Some(value) } else { None }
    }

    #[inline(always)]
    fn to_f32(self) -> f32 {
        self as f32
    }

    #[inline(always)]
    fn from_f32(value: f32) -> Self {
        if value.is_infinite() {
            if value.is_sign_negative() { f64::MIN } else { f64::MAX }
        } else if value.is_nan() {
            0.0
        } else {
            value as f64
        }
    }
}

impl QuantizableValue for f32 {
    const NUMBER_OF_BYTES: usize = size_of::<f32>();
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;
    const MAX_VALUE: Self = f32::MAX;
    const MIN_VALUE: Self = f32::MIN;

    #[inline(always)]
    fn saturating_add(self, other: Self) -> Self {
        let value = self + other;
        if value.is_infinite() {
            if value.is_sign_negative() { f32::MIN } else { f32::MAX }
        } else if value.is_nan() {
            0.0
        } else {
            value
        }
    }

    #[inline(always)]
    fn checked_add(self, other: Self) -> Option<Self> {
        let value = self + other;
        if value.is_finite() { Some(value) } else { None }
    }

    #[inline(always)]
    fn saturating_sub(self, other: Self) -> Self {
        let value = self - other;
        if value.is_infinite() {
            if value.is_sign_negative() { f32::MIN } else { f32::MAX }
        } else if value.is_nan() {
            0.0
        } else {
            value
        }
    }

    #[inline(always)]
    fn checked_sub(self, other: Self) -> Option<Self> {
        let value = self - other;
        if value.is_finite() { Some(value) } else { None }
    }

    #[inline(always)]
    fn saturating_mul(self, other: Self) -> Self {
        let value = self * other;
        if value.is_infinite() {
            if value.is_sign_negative() { f32::MIN } else { f32::MAX }
        } else if value.is_nan() {
            0.0
        } else {
            value
        }
    }

    #[inline(always)]
    fn checked_mul(self, other: Self) -> Option<Self> {
        let value = self * other;
        if value.is_finite() { Some(value) } else { None }
    }

    #[inline(always)]
    fn checked_div(self, other: Self) -> Option<Self> {
        let value = self / other;
        if value.is_finite() { Some(value) } else { None }
    }

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

impl QuantizableValue for f16 {
    const NUMBER_OF_BYTES: usize = size_of::<f16>();
    const ZERO: Self = f16::ZERO;
    const ONE: Self = f16::ONE;
    const MAX_VALUE: Self = f16::MAX;
    const MIN_VALUE: Self = f16::MIN;

    #[inline(always)]
    fn saturating_add(self, other: Self) -> Self {
        Self::from_f32(self.to_f32() + other.to_f32())
    }

    #[inline(always)]
    fn checked_add(self, other: Self) -> Option<Self> {
        let value = self.to_f32() + other.to_f32();
        if value.is_finite() && value <= f16::MAX.to_f32() && value >= f16::MIN.to_f32() {
            Some(f16::from_f32(value))
        } else {
            None
        }
    }

    #[inline(always)]
    fn saturating_sub(self, other: Self) -> Self {
        Self::from_f32(self.to_f32() - other.to_f32())
    }

    #[inline(always)]
    fn checked_sub(self, other: Self) -> Option<Self> {
        let value = self.to_f32() - other.to_f32();
        if value.is_finite() && value <= f16::MAX.to_f32() && value >= f16::MIN.to_f32() {
            Some(f16::from_f32(value))
        } else {
            None
        }
    }

    #[inline(always)]
    fn saturating_mul(self, other: Self) -> Self {
        Self::from_f32(self.to_f32() * other.to_f32())
    }

    #[inline(always)]
    fn checked_mul(self, other: Self) -> Option<Self> {
        let value = self.to_f32() * other.to_f32();
        if value.is_finite() && value <= f16::MAX.to_f32() && value >= f16::MIN.to_f32() {
            Some(f16::from_f32(value))
        } else {
            None
        }
    }

    #[inline(always)]
    fn checked_div(self, other: Self) -> Option<Self> {
        let value = self.to_f32() / other.to_f32();
        if value.is_finite() && value <= f16::MAX.to_f32() && value >= f16::MIN.to_f32() {
            Some(f16::from_f32(value))
        } else {
            None
        }
    }

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

impl QuantizableValue for u8 {
    const NUMBER_OF_BYTES: usize = size_of::<u8>();
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const MAX_VALUE: Self = u8::MAX;
    const MIN_VALUE: Self = u8::MIN;

    #[inline(always)]
    fn saturating_add(self, other: Self) -> Self {
        self.saturating_add(other)
    }

    #[inline(always)]
    fn checked_add(self, other: Self) -> Option<Self> {
        self.checked_add(other)
    }

    #[inline(always)]
    fn saturating_sub(self, other: Self) -> Self {
        self.saturating_sub(other)
    }

    #[inline(always)]
    fn checked_sub(self, other: Self) -> Option<Self> {
        self.checked_sub(other)
    }

    #[inline(always)]
    fn saturating_mul(self, other: Self) -> Self {
        self.saturating_mul(other)
    }

    #[inline(always)]
    fn checked_mul(self, other: Self) -> Option<Self> {
        self.checked_mul(other)
    }

    #[inline(always)]
    fn checked_div(self, other: Self) -> Option<Self> {
        self.checked_div(other)
    }

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
