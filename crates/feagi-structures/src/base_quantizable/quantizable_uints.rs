use core::fmt::{Debug, Display};

//region UInts

#[cfg(not(feature = "alloc"))]
pub trait QuantizableUInt:
    Copy + Clone + Send + Sync + Into<usize> + 'static
{
    const NUMBER_OF_BYTES: usize;
    fn ge(self, other: Self) -> bool;
    fn lt(self, other: Self) -> bool;
    fn zero() -> Self;
    fn one() -> Self;
    fn max_value() -> Self;
    fn min_value() -> Self;
}

#[cfg(feature = "alloc")]
pub trait QuantizableUInt:
    Copy + Clone + Send + Sync + Debug + Display + Default + Into<usize>  + 'static
{
    const NUMBER_OF_BYTES: usize;
    fn ge(self, other: Self) -> bool;
    fn lt(self, other: Self) -> bool;
    fn zero() -> Self;
    fn one() -> Self;
    fn max_value() -> Self;
    fn min_value() -> Self;
}

impl QuantizableUInt for u8 {
    const NUMBER_OF_BYTES: usize = size_of::<u8>();

    #[inline(always)]
    fn ge(self, other: Self) -> bool {
        self >= other
    }

    #[inline(always)]
    fn lt(self, other: Self) -> bool {
        self < other
    }

    #[inline(always)]
    fn zero() -> Self {
        0
    }

    #[inline(always)]
    fn one() -> Self {
        1
    }

    #[inline(always)]
    fn max_value() -> Self {
        u8::MAX
    }

    #[inline(always)]
    fn min_value() -> Self {
        u8::MIN
    }
}

impl Into<usize> for u8 {
    fn into(self) -> usize {
        self as usize  // Just a cast - no computation
    }
}

impl QuantizableUInt for u16 {
    const NUMBER_OF_BYTES: usize = size_of::<u16>();

    #[inline(always)]
    fn ge(self, other: Self) -> bool {
        self >= other
    }

    #[inline(always)]
    fn lt(self, other: Self) -> bool {
        self < other
    }

    #[inline(always)]
    fn zero() -> Self {
        0
    }

    #[inline(always)]
    fn one() -> Self {
        1
    }

    #[inline(always)]
    fn max_value() -> Self {
        u16::MAX
    }

    #[inline(always)]
    fn min_value() -> Self {
        u16::MIN
    }
}

impl Into<usize> for u16 {
    fn into(self) -> usize {
        self as usize  // Just a cast - no computation
    }
}

impl QuantizableUInt for u32 {
    const NUMBER_OF_BYTES: usize = size_of::<u32>();

    #[inline(always)]
    fn ge(self, other: Self) -> bool {
        self >= other
    }

    #[inline(always)]
    fn lt(self, other: Self) -> bool {
        self < other
    }

    #[inline(always)]
    fn zero() -> Self {
        0
    }

    #[inline(always)]
    fn one() -> Self {
        1
    }

    #[inline(always)]
    fn max_value() -> Self {
        u32::MAX
    }

    #[inline(always)]
    fn min_value() -> Self {
        u32::MIN
    }
}

impl Into<usize> for u32 {
    fn into(self) -> usize {
        self as usize  // Just a cast - no computation
    }
}

impl QuantizableUInt for u64 {
    const NUMBER_OF_BYTES: usize = size_of::<u64>();

    #[inline(always)]
    fn ge(self, other: Self) -> bool {
        self >= other
    }

    #[inline(always)]
    fn lt(self, other: Self) -> bool {
        self < other
    }

    #[inline(always)]
    fn zero() -> Self {
        0
    }

    #[inline(always)]
    fn one() -> Self {
        1
    }

    #[inline(always)]
    fn max_value() -> Self {
        u64::MAX
    }

    #[inline(always)]
    fn min_value() -> Self {
        u64::MIN
    }
}

impl Into<usize> for u64 {
    fn into(self) -> usize {
        self as usize  // Just a cast - no computation
    }
}

//endregion
