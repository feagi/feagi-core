use core::fmt::{Debug, Display};

//region Ints

#[cfg(not(feature = "alloc"))]
pub trait QuantizableInt:
    Copy + Clone + Send + Sync + 'static
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
pub trait QuantizableInt:
    Copy + Clone + Send + Sync + Debug + Display + Default + Into<isize> + 'static
{
    const NUMBER_OF_BYTES: usize;
    fn ge(self, other: Self) -> bool;
    fn lt(self, other: Self) -> bool;
    fn zero() -> Self;
    fn one() -> Self;
    fn max_value() -> Self;
    fn min_value() -> Self;
}

impl QuantizableInt for isize {
    const NUMBER_OF_BYTES: usize = size_of::<isize>();

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
        isize::MAX
    }

    #[inline(always)]
    fn min_value() -> Self {
        isize::MIN
    }
}

impl Into<isize> for isize {
    fn into(self) -> isize { self } // lol
}


impl QuantizableInt for i8 {
    const NUMBER_OF_BYTES: usize = size_of::<i8>();

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
        i8::MAX
    }

    #[inline(always)]
    fn min_value() -> Self {
        i8::MIN
    }
}

impl Into<isize> for i8 {
    fn into(self) -> isize { self as isize} // lol
}

impl QuantizableInt for i16 {
    const NUMBER_OF_BYTES: usize = size_of::<i16>();

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
        i16::MAX
    }

    #[inline(always)]
    fn min_value() -> Self {
        i16::MIN
    }
}

impl Into<isize> for i16 {
    fn into(self) -> isize { self as isize } // lol
}

impl QuantizableInt for i32 {
    const NUMBER_OF_BYTES: usize = size_of::<i32>();

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
        i32::MAX
    }

    #[inline(always)]
    fn min_value() -> Self {
        i32::MIN
    }
}

impl Into<isize> for i32 {
    fn into(self) -> isize { self as isize } // lol
}

impl QuantizableInt for i64 {
    const NUMBER_OF_BYTES: usize = size_of::<i64>();

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
        i64::MAX
    }

    #[inline(always)]
    fn min_value() -> Self {
        i64::MIN
    }
}

impl Into<isize> for i64 {
    fn into(self) -> isize { self as isize } // lol
}

//endregion
