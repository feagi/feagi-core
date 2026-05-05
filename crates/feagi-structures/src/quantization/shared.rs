//! Defines common base traits shared among many quantization traits

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
+ 'static
{
    const NUMBER_OF_BYTES: usize;

    fn saturating_add(self, other: Self) -> Self;
    fn checked_add(self, other: Self) -> Option<Self>;
    fn saturating_sub(self, other: Self) -> Self;
    fn checked_sub(self, other: Self) -> Option<Self>;
    fn saturating_mul(self, other: Self) -> Self;
    fn checked_mul(self, other: Self) -> Option<Self>;
    // No need for saturating div
    fn checked_div(self, other: Self) -> Option<Self>;
}

/// Defines a single Quantizable element (a single number)
pub trait FeagiBaseSingleElementQuantizationType: FeagiBaseQuantizationType
+ core::cmp::PartialOrd
{
    const ZERO: Self;
    const ONE: Self;
    const MAX_VALUE: Self;
    const MIN_VALUE: Self;
}

/// Defines a collection of Quantizable elements of the same type
pub trait FeagiBaseMultiElementQuantizationType: FeagiBaseQuantizationType
{
    const NUMBER_ELEMENTS: usize;
    const NUMBER_OF_BYTES: usize = Self::NUMBER_ELEMENTS * Self::ElementType::NUMBER_OF_BYTES;
    const ALL_ZEROS: Self;
    const ALL_ONES: Self;
    type ElementType: FeagiBaseSingleElementQuantizationType;
}
