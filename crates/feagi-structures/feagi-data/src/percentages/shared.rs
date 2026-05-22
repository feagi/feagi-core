#[cfg(not(feature = "alloc"))]
pub trait FeagiBasePercentageType:
Copy
+ Clone
+ Send
+ Sync
+ core::cmp::Eq
+ core::hash::Hash
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
    const ZERO_PERCENT: Self;
    const HUNDRED_PERCENT: Self;
    const MAX_AS_F32: f32;

    fn saturating_add(self, other: Self) -> Self;
    fn checked_add(self, other: Self) -> Option<Self>;
    fn saturating_sub(self, other: Self) -> Self;
    fn checked_sub(self, other: Self) -> Option<Self>;
    fn saturating_mul(self, other: Self) -> Self;
    fn checked_mul(self, other: Self) -> Option<Self>;
    fn checked_div(self, other: Self) -> Option<Self>;
    fn from_f32(float: f32) -> Option<Self>;
    fn to_f32(self) -> f32;
}


#[cfg(feature = "alloc")]
pub trait FeagiBasePercentageType:
Copy
+ Clone
+ Send
+ Sync
+ core::cmp::Eq
+ core::hash::Hash
+ core::cmp::PartialOrd
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
    const ZERO_PERCENT: Self;
    const HUNDRED_PERCENT: Self;
    const MAX_AS_F32: f32;

    fn saturating_add(self, other: Self) -> Self;
    fn checked_add(self, other: Self) -> Option<Self>;
    fn saturating_sub(self, other: Self) -> Self;
    fn checked_sub(self, other: Self) -> Option<Self>;
    fn saturating_mul(self, other: Self) -> Self;
    fn checked_mul(self, other: Self) -> Option<Self>;
    fn checked_div(self, other: Self) -> Option<Self>;
    fn try_from_f32(float: f32) -> Option<Self>;
    fn to_f32(self) -> f32;
}

// TODO remove option implementations for hardware
