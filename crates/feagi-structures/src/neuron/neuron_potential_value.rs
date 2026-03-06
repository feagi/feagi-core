
// TODO more proper implementation
// TODO think about this carefully

#[cfg(feature = "alloc")]
use core::fmt::{Debug, Display};

pub type NeuronPotentialF32 = f32;

#[cfg(not(feature = "alloc"))]
pub trait NeuralPotentialValue:
    Copy + Clone + Send + Sync + 'static
{
    const NUMBER_OF_BYTES: usize;
    fn from_f32(value: f32) -> Self;
    fn to_f32(self) -> f32;
    fn saturating_add(self, other: Self) -> Self;
    fn mul_leak(self, leak_coefficient: f32) -> Self;
    fn ge(self, other: Self) -> bool;
    fn lt(self, other: Self) -> bool;
    fn zero() -> Self;
    fn one() -> Self;
    fn max_value() -> Self;
    fn min_value() -> Self;
}

#[cfg(feature = "alloc")]
pub trait NeuralPotentialValue:
Copy + Clone + Send + Sync + Debug + Display + Default + 'static
{
    const NUMBER_OF_BYTES: usize;
    fn from_f32(value: f32) -> Self;
    fn to_f32(self) -> f32;
    fn saturating_add(self, other: Self) -> Self;
    fn mul_leak(self, leak_coefficient: f32) -> Self;
    fn ge(self, other: Self) -> bool;
    fn lt(self, other: Self) -> bool;
    fn zero() -> Self;
    fn one() -> Self;
    fn max_value() -> Self;
    fn min_value() -> Self;
}


impl NeuralPotentialValue for f32 {
    const NUMBER_OF_BYTES: usize = size_of::<f32>();

    #[inline(always)]
    fn from_f32(value: f32) -> Self {
        value
    }

    #[inline(always)]
    fn to_f32(self) -> f32 {
        self
    }

    #[inline(always)]
    fn saturating_add(self, other: Self) -> Self {
        self + other
    }

    #[inline(always)]
    fn mul_leak(self, leak_coefficient: f32) -> Self {
        self * (1.0 - leak_coefficient)
    }

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

