//! A Trait that simply requires those that implement to support basic number and math operations.

use half::f16;
use crate::quantizable_linear::custom_data_types::StorageF8;

#[cfg(not(feature = "alloc"))]
/// Supports basic core / math operations
pub trait SupportsBasicCoreMathOps:
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
+ Sized

+ 'static
{

}

#[cfg(feature = "alloc")]
/// Supports basic core / math operations
pub trait SupportsBasicCoreMathOps:
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
+ Sized

// Alloc variant supports Debug and Display
+ core::fmt::Debug
+ core::fmt::Display
+ 'static
{

}

//region Boring Implementations

impl SupportsBasicCoreMathOps for usize {}

impl SupportsBasicCoreMathOps for u8 {}

impl SupportsBasicCoreMathOps for u16 {}

impl SupportsBasicCoreMathOps for u32 {

}

#[cfg(feature = "support_64bit_indexing")]
impl SupportsBasicCoreMathOps for u64 {
}

// Lol no we are not doing u128 or i128

impl SupportsBasicCoreMathOps for isize {}

impl SupportsBasicCoreMathOps for i8 { }

impl SupportsBasicCoreMathOps for i16 { }

impl SupportsBasicCoreMathOps for i32 { }

#[cfg(feature = "support_64bit_indexing")]
impl SupportsBasicCoreMathOps for i64 { }

// A bad choice for computation
impl SupportsBasicCoreMathOps for StorageF8 { }

impl SupportsBasicCoreMathOps for f16 { }

impl SupportsBasicCoreMathOps for f32 { }

#[cfg(feature = "support_64bit_values")]
impl SupportsBasicCoreMathOps for f64 { }


//endregion


// Supports uint operations (remainder, usize conversions)
pub trait SupportsUintOps:
SupportsBasicCoreMathOps
+ core::ops::Rem<Output = Self>
+ core::ops::RemAssign
+ core::cmp::Eq
+ core::hash::Hash
{
    const QUANT_MAX: Self;
    const QUANT_MAX_AS_USIZE: usize;

    const QUANT_ONE: Self;

    /// will convert from usize to self type without checks, which if outside the range could
    /// crash or cause unexpected behavior. However this is fast
    fn from_usize_unchecked(u: usize) -> Self;

    /// Will first check if the usize is in range, if not it will clamp to the max value.
    /// Note this is slower
    fn from_usize_clamped(u: usize) -> Self;

    fn to_usize(self) -> usize;

}

//region Boring Implementations

// lol
impl SupportsUintOps for usize {
    const QUANT_MAX: Self = core::usize::MAX;
    const QUANT_MAX_AS_USIZE: usize = usize::MAX;
    const QUANT_ONE: Self = 1;

    #[inline(always)]
    fn from_usize_unchecked(u: usize) -> Self {
        u
    }

    #[inline(always)]
    fn from_usize_clamped(u: usize) -> Self {
        u
    }

    #[inline(always)]
    fn to_usize(self) -> usize {
        self
    }
}

impl SupportsUintOps for u8 {
    const QUANT_MAX: Self = core::u8::MAX;
    const QUANT_MAX_AS_USIZE: usize = u8::MAX as usize;
    const QUANT_ONE: Self = 1;

    #[inline(always)]
    fn from_usize_unchecked(u: usize) -> Self {
        u as u8
    }

    #[inline(always)]
    fn from_usize_clamped(u: usize) -> Self {
        if u > Self::QUANT_MAX_AS_USIZE {
            return u8::MAX
        }
        u as u8
    }

    #[inline(always)]
    fn to_usize(self) -> usize {
        self as usize
    }
}

impl SupportsUintOps for u16 {
    const QUANT_MAX: Self = core::u16::MAX;
    const QUANT_MAX_AS_USIZE: usize = u16::MAX as usize;
    const QUANT_ONE: Self = 1;

    #[inline(always)]
    fn from_usize_unchecked(u: usize) -> Self {
        u as u16
    }

    #[inline(always)]
    fn from_usize_clamped(u: usize) -> Self {
        if u > Self::QUANT_MAX_AS_USIZE {
            return u16::MAX
        }
        u as u16
    }

    #[inline(always)]
    fn to_usize(self) -> usize {
        self as usize
    }
}

impl SupportsUintOps for u32 {
    const QUANT_MAX: Self = core::u32::MAX;
    const QUANT_MAX_AS_USIZE: usize = u32::MAX as usize;
    const QUANT_ONE: Self = 1;

    #[inline(always)]
    fn from_usize_unchecked(u: usize) -> Self {
        u as u32
    }

    #[inline(always)]
    fn from_usize_clamped(u: usize) -> Self {
        if u > Self::QUANT_MAX_AS_USIZE {
            return u32::MAX
        }
        u as u32
    }

    #[inline(always)]
    fn to_usize(self) -> usize {
        self as usize
    }
}

#[cfg(feature = "support_64bit_indexing")]
impl SupportsUintOps for u64 {
    const QUANT_MAX: Self = core::u64::MAX;
    const QUANT_MAX_AS_USIZE: usize = u64::MAX as usize;
    const QUANT_ONE: Self = 1;

    #[inline(always)]
    fn from_usize_unchecked(u: usize) -> Self {
        u as u64
    }

    #[inline(always)]
    fn from_usize_clamped(u: usize) -> Self {
        if u > Self::MAX_AS_USIZE {
            return u64::MAX
        }
        u as u64
    }

    #[inline(always)]
    fn to_usize(self) -> usize {
        self as usize
    }
}

//endregion