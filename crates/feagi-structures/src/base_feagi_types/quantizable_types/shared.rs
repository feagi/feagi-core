//! Defines common base traits shared among many quantization traits

use half::f16;
use crate::quantization_level::QuantizationLevel;

/// Common base for all quantizable types (Alloc methods disabled)
#[cfg(not(feature = "alloc"))]
pub trait FeagiBaseQuantizationType:
Copy
+ Clone
+ Send
+ Sync
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
    const NUMBER_OF_BYTES: usize;

    fn saturating_add(self, other: Self) -> Self;
    fn checked_add(self, other: Self) -> Option<Self>;
    fn saturating_sub(self, other: Self) -> Self;
    fn checked_sub(self, other: Self) -> Option<Self>;
    fn saturating_mul(self, other: Self) -> Self;
    fn checked_mul(self, other: Self) -> Option<Self>;
    fn checked_div(self, other: Self) -> Option<Self>;
}

/// Common base for all quantizable types (Alloc methods enabled)
#[cfg(feature = "alloc")]
pub trait FeagiBaseQuantizationType:
Copy
+ Clone
+ Send
+ Sync
+ core::fmt::Debug
+ core::fmt::Display
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
    const NUMBER_OF_BYTES: usize;
    const QUANTIZATION_LEVEL: QuantizationLevel;

    fn saturating_add(self, other: Self) -> Self;
    fn checked_add(self, other: Self) -> Option<Self>;
    fn saturating_sub(self, other: Self) -> Self;
    fn checked_sub(self, other: Self) -> Option<Self>;
    fn saturating_mul(self, other: Self) -> Self;
    fn checked_mul(self, other: Self) -> Option<Self>;
    // No need for saturating div
    fn checked_div(self, other: Self) -> Option<Self>;
}

impl FeagiBaseQuantizationType for usize {
    const NUMBER_OF_BYTES: usize = size_of::<usize>();
    const QUANTIZATION_LEVEL: QuantizationLevel = QuantizationLevel::Bit32; // TODO: 64


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
}

impl FeagiBaseQuantizationType for u8 {
    const NUMBER_OF_BYTES: usize = size_of::<u8>();
    const QUANTIZATION_LEVEL: QuantizationLevel = QuantizationLevel::Bit8;

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
}

impl FeagiBaseQuantizationType for u16 {
    const NUMBER_OF_BYTES: usize = size_of::<u16>();
    const QUANTIZATION_LEVEL: QuantizationLevel = QuantizationLevel::Bit16;

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
}

impl FeagiBaseQuantizationType for u32 {
    const NUMBER_OF_BYTES: usize = size_of::<u32>();
    const QUANTIZATION_LEVEL: QuantizationLevel = QuantizationLevel::Bit32;

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
}

#[cfg(feature = "support_64bit_indexing")]
impl FeagiBaseQuantizationType for u64 {
    const NUMBER_OF_BYTES: usize = size_of::<u64>();
    const QUANTIZATION_LEVEL: QuantizationLevel = QuantizationLevel::Bit64;

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
}



impl FeagiBaseQuantizationType for isize {
    const NUMBER_OF_BYTES: usize = size_of::<isize>();
    const QUANTIZATION_LEVEL: QuantizationLevel = QuantizationLevel::Bit32; // TODO 64

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
}

impl FeagiBaseQuantizationType for i8 {
    const NUMBER_OF_BYTES: usize = size_of::<i8>();
    const QUANTIZATION_LEVEL: QuantizationLevel = QuantizationLevel::Bit8;

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
}

impl FeagiBaseQuantizationType for i16 {
    const NUMBER_OF_BYTES: usize = size_of::<i16>();
    const QUANTIZATION_LEVEL: QuantizationLevel = QuantizationLevel::Bit16;

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
}

impl FeagiBaseQuantizationType for i32 {
    const NUMBER_OF_BYTES: usize = size_of::<i32>();
    const QUANTIZATION_LEVEL: QuantizationLevel = QuantizationLevel::Bit32;

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
}



#[cfg(feature = "support_64bit_indexing")]
impl FeagiBaseQuantizationType for i64 {
    const NUMBER_OF_BYTES: usize = size_of::<i64>();
    const QUANTIZATION_LEVEL: QuantizationLevel = QuantizationLevel::Bit64;

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
}

impl FeagiBaseQuantizationType for f16 {
    const NUMBER_OF_BYTES: usize = size_of::<f16>();
    const QUANTIZATION_LEVEL: QuantizationLevel = QuantizationLevel::Bit16;

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
}

impl FeagiBaseQuantizationType for f32 {
    const NUMBER_OF_BYTES: usize = size_of::<f32>();
    const QUANTIZATION_LEVEL: QuantizationLevel = QuantizationLevel::Bit32;

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
}

#[cfg(feature = "support_64bit_values")]
impl FeagiBaseQuantizationType for f64 {
    const NUMBER_OF_BYTES: usize = size_of::<f64>();
    const QUANTIZATION_LEVEL: QuantizationLevel = QuantizationLevel::Bit64;

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
}
/// Defines a single Quantizable element (a single number)
pub trait FeagiBaseSingleElementQuantizationType: FeagiBaseQuantizationType
{
    const ZERO: Self;
    const ONE: Self;
    const MAX_VALUE: Self;
    const MIN_VALUE: Self;
}
impl FeagiBaseSingleElementQuantizationType for usize {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const MAX_VALUE: Self = usize::MAX;
    const MIN_VALUE: Self = usize::MIN;
}
impl FeagiBaseSingleElementQuantizationType for u8 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const MAX_VALUE: Self = u8::MAX;
    const MIN_VALUE: Self = u8::MIN;
}

impl FeagiBaseSingleElementQuantizationType for u16 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const MAX_VALUE: Self = u16::MAX;
    const MIN_VALUE: Self = u16::MIN;
}

impl FeagiBaseSingleElementQuantizationType for u32 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const MAX_VALUE: Self = u32::MAX;
    const MIN_VALUE: Self = u32::MIN;
}

#[cfg(feature = "support_64bit_indexing")]
impl FeagiBaseSingleElementQuantizationType for u64 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const MAX_VALUE: Self = u64::MAX;
    const MIN_VALUE: Self = u64::MIN;
}


impl FeagiBaseSingleElementQuantizationType for f16 {
    const ZERO: Self = f16::ZERO;
    const ONE: Self = f16::ONE;
    const MAX_VALUE: Self = f16::MAX;
    const MIN_VALUE: Self = f16::MIN;
}

impl FeagiBaseSingleElementQuantizationType for f32 {
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;
    const MAX_VALUE: Self = f32::MAX;
    const MIN_VALUE: Self = f32::MIN;
}

#[cfg(feature = "support_64bit_values")]
impl FeagiBaseSingleElementQuantizationType for f64 {
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;
    const MAX_VALUE: Self = f64::MAX;
    const MIN_VALUE: Self = f64::MIN;
}


impl FeagiBaseSingleElementQuantizationType for isize {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const MAX_VALUE: Self = isize::MAX;
    const MIN_VALUE: Self = isize::MIN;
}

impl FeagiBaseSingleElementQuantizationType for i8 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const MAX_VALUE: Self = i8::MAX;
    const MIN_VALUE: Self = i8::MIN;
}

impl FeagiBaseSingleElementQuantizationType for i16 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const MAX_VALUE: Self = i16::MAX;
    const MIN_VALUE: Self = i16::MIN;
}

impl FeagiBaseSingleElementQuantizationType for i32 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const MAX_VALUE: Self = i32::MAX;
    const MIN_VALUE: Self = i32::MIN;
}

#[cfg(feature = "support_64bit_indexing")]
impl FeagiBaseSingleElementQuantizationType for i64 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const MAX_VALUE: Self = i64::MAX;
    const MIN_VALUE: Self = i64::MIN;
}



/// Defines a collection of Quantizable elements of the same type
pub trait FeagiBaseMultiElementQuantizationType: FeagiBaseQuantizationType
{
    const NUMBER_ELEMENTS: usize;
    const NUMBER_OF_BYTES: usize = Self::NUMBER_ELEMENTS * Self::ElementType::NUMBER_OF_BYTES;
    type ElementType: FeagiBaseSingleElementQuantizationType;
}
