use core::fmt::{Debug, Display};

//region Floats

#[cfg(not(feature = "alloc"))]
pub trait QuantizableFloat:
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
pub trait QuantizableFloat:
Copy + Clone + Send + Sync + Debug + Display + Default + 'static
{
    const NUMBER_OF_BYTES: usize;
    fn ge(self, other: Self) -> bool;
    fn lt(self, other: Self) -> bool;
    fn zero() -> Self;
    fn one() -> Self;
    fn max_value() -> Self;
    fn min_value() -> Self;
}

impl QuantizableFloat for f32 {
    const NUMBER_OF_BYTES: usize = size_of::<f32>();

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
        0.0
    }

    #[inline(always)]
    fn one() -> Self {
        1.0
    }

    #[inline(always)]
    fn max_value() -> Self {
        f32::MAX
    }

    #[inline(always)]
    fn min_value() -> Self {
        f32::MIN
    }
}

impl QuantizableFloat for f64 {
    const NUMBER_OF_BYTES: usize = size_of::<f64>();

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
        0.0
    }

    #[inline(always)]
    fn one() -> Self {
        1.0
    }

    #[inline(always)]
    fn max_value() -> Self {
        f64::MAX
    }

    #[inline(always)]
    fn min_value() -> Self {
        f64::MIN
    }
}

 //endregion

