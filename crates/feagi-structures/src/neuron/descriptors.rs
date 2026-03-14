use core::fmt::{Debug, Display};
use crate::base_quantizable::coordinate::UnsignedCoordinate3DType;
use crate::base_quantizable::unsigned_integer::QuantizableUInt;
use crate::base_quantizable::value::QuantizableValue;

pub type NeuronVoxelCoordinate<T: QuantizableUInt> = UnsignedCoordinate3DType<T>;

//region Neuron ID
pub type NeuronIdType<T> = T;
pub type NeuronIdU64 = NeuronIdType<u64>;
pub type NeuronIdU32 = NeuronIdType<u32>;
pub type NeuronIdU16 = NeuronIdType<u16>;
pub type NeuronIdU8 = NeuronIdType<u8>;

#[cfg(not(feature = "alloc"))]
pub trait NeuronId:
Copy + Clone + Send + Sync + QuantizableUInt + 'static
{

}

#[cfg(feature = "alloc")]
pub trait NeuronId:
Copy + Clone + Send + Sync + Debug + Display + Default + QuantizableUInt + 'static
{

}

impl NeuronId for u64 {

}

impl NeuronId for u32 {

}

impl NeuronId for u16 {

}

impl NeuronId for u8 {

}

//endregion

//region Neuron Count
pub type NeuronCountType<T> = T;
pub type NeuronCountU64 = NeuronCountType<u64>;
pub type NeuronCountU32 = NeuronCountType<u32>;
pub type NeuronCountU16 = NeuronCountType<u16>;
pub type NeuronCountU8 = NeuronCountType<u8>;

#[cfg(not(feature = "alloc"))]
pub trait NeurCountId:
Copy + Clone + Send + Sync + QuantizableUInt + 'static
{

}

#[cfg(feature = "alloc")]
pub trait NeuronCount:
Copy + Clone + Send + Sync + Debug + Display + Default + QuantizableUInt + 'static
{

}

impl NeuronCount for u64 {

}

impl NeuronCount for u32 {

}

impl NeuronCount for u16 {

}

impl NeuronCount for u8 {

}

//endregion

//region Neuron Potential Value

pub type NeuronPotentialF32 = f32;

#[cfg(not(feature = "alloc"))]
pub trait NeuralPotentialValue:
Copy + Clone + Send + Sync + QuantizableValue + 'static
{
    fn from_f32(value: f32) -> Self;
    fn to_f32(self) -> f32;
    fn saturating_add(self, other: Self) -> Self;
    fn mul_leak(self, leak_coefficient: f32) -> Self;
}

#[cfg(feature = "alloc")]
pub trait NeuralPotentialValue:
Copy + Clone + Send + Sync + Debug + Display + Default + QuantizableValue + 'static
{
    fn from_f32(value: f32) -> Self;
    fn to_f32(self) -> f32;
    fn saturating_add(self, other: Self) -> Self;
    fn mul_leak(self, leak_coefficient: f32) -> Self;
}


impl NeuralPotentialValue for f32 {

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
}

//endregion

//region Leak Coefficient

pub type LeakCoefficientF32 = f32;

#[cfg(not(feature = "alloc"))]
pub trait LeakCoefficientF32:
Copy + Clone + Send + Sync + QuantizableValue + 'static
{
    fn from_f32(value: f32) -> Self;
    fn to_f32(self) -> f32;
    fn saturating_add(self, other: Self) -> Self;
    fn mul_leak(self, leak_coefficient: f32) -> Self;
}

#[cfg(feature = "alloc")]
pub trait LeakCoefficient:
Copy + Clone + Send + Sync + Debug + Display + Default + QuantizableValue + 'static
{
    fn from_f32(value: f32) -> Self;
    fn to_f32(self) -> f32;
}


impl LeakCoefficient for f32 {

    #[inline(always)]
    fn from_f32(value: f32) -> Self {
        value
    }

    #[inline(always)]
    fn to_f32(self) -> f32 {
        self
    }
}

//endregion

//region Refractory Period

pub type RefractoryPeriodType<T> = T;
pub type RefractoryPeriodU64 = RefractoryPeriodType<u64>;
pub type RefractoryPeriodU32 = RefractoryPeriodType<u32>;
pub type RefractoryPeriodU16 = RefractoryPeriodType<u16>;
pub type RefractoryPeriodU8 = RefractoryPeriodType<u8>;

#[cfg(not(feature = "alloc"))]
pub trait RefractoryPeriod:
Copy + Clone + Send + Sync + QuantizableUInt + 'static
{

}

#[cfg(feature = "alloc")]
pub trait RefractoryPeriod:
Copy + Clone + Send + Sync + Debug + Display + Default + QuantizableUInt + 'static
{

}

impl RefractoryPeriod for u64 {

}

impl RefractoryPeriod for u32 {

}

impl RefractoryPeriod for u16 {

}

impl RefractoryPeriod for u8 {

}

//endregion

//region Refractory Countdown

pub type RefractoryCountdownType<T> = T;
pub type RefractoryCountdownU64 = RefractoryCountdownType<u64>;
pub type RefractoryCountdownU32 = RefractoryCountdownType<u32>;
pub type RefractoryCountdownU16 = RefractoryCountdownType<u16>;
pub type RefractoryCountdownU8 = RefractoryCountdownType<u8>;

#[cfg(not(feature = "alloc"))]
pub trait RefractoryCountdown:
Copy + Clone + Send + Sync + QuantizableUInt + 'static
{

}

#[cfg(feature = "alloc")]
pub trait RefractoryCountdown:
Copy + Clone + Send + Sync + Debug + Display + Default + QuantizableUInt + 'static
{

}

impl RefractoryCountdown for u64 {

}

impl RefractoryCountdown for u32 {

}

impl RefractoryCountdown for u16 {

}

impl RefractoryCountdown for u8 {

}

//endregion

//region Excitability

// TODO this may be better as a percentage?
pub type ExcitabilityF32 = f32;

#[cfg(not(feature = "alloc"))]
pub trait Excitability:
Copy + Clone + Send + Sync + QuantizableValue + 'static
{
    fn from_f32(value: f32) -> Self;
    fn to_f32(self) -> f32;
    fn saturating_add(self, other: Self) -> Self;
    fn mul_leak(self, leak_coefficient: f32) -> Self;
}

#[cfg(feature = "alloc")]
pub trait Excitability:
Copy + Clone + Send + Sync + Debug + Display + Default + QuantizableValue + 'static
{

}


impl Excitability for f32 {
    
}

//endregion

//region Consecutive Fire Countdown

pub type ConsecutiveFireCountdownType<T> = T;
pub type ConsecutiveFireCountdownU64 = ConsecutiveFireCountdownType<u64>;
pub type ConsecutiveFireCountdownU32 = ConsecutiveFireCountdownType<u32>;
pub type ConsecutiveFireCountdownU16 = ConsecutiveFireCountdownType<u16>;
pub type ConsecutiveFireCountdownU8 = ConsecutiveFireCountdownType<u8>;

#[cfg(not(feature = "alloc"))]
pub trait ConsecutiveFireCountdown:
Copy + Clone + Send + Sync + QuantizableUInt + 'static
{

}

#[cfg(feature = "alloc")]
pub trait ConsecutiveFireCountdown:
Copy + Clone + Send + Sync + Debug + Display + Default + QuantizableUInt + 'static
{

}

impl ConsecutiveFireCountdown for u64 {

}

impl ConsecutiveFireCountdown for u32 {

}

impl ConsecutiveFireCountdown for u16 {

}

impl ConsecutiveFireCountdown for u8 {

}

//endregion

//region Consecutive Fire Limit

pub type ConsecutiveFireLimitType<T> = T;
pub type ConsecutiveFireLimitU64 = ConsecutiveFireLimitType<u64>;
pub type ConsecutiveFireLimitU32 = ConsecutiveFireLimitType<u32>;
pub type ConsecutiveFireLimitU16 = ConsecutiveFireLimitType<u16>;
pub type ConsecutiveFireLimitU8 = ConsecutiveFireLimitType<u8>;

#[cfg(not(feature = "alloc"))]
pub trait ConsecutiveFireLimit:
Copy + Clone + Send + Sync + QuantizableUInt + 'static
{

}

#[cfg(feature = "alloc")]
pub trait ConsecutiveFireLimit:
Copy + Clone + Send + Sync + Debug + Display + Default + QuantizableUInt + 'static
{

}

impl ConsecutiveFireLimit for u64 {

}

impl ConsecutiveFireLimit for u32 {

}

impl ConsecutiveFireLimit for u16 {

}

impl ConsecutiveFireLimit for u8 {

}

//endregion

//region Snooze Period Countdown

pub type SnoozePeriodCountdownType<T> = T;
pub type SnoozePeriodCountdownU64 = SnoozePeriodCountdownType<u64>;
pub type SnoozePeriodCountdownU32 = SnoozePeriodCountdownType<u32>;
pub type SnoozePeriodCountdownU16 = SnoozePeriodCountdownType<u16>;
pub type SnoozePeriodCountdownU8 = SnoozePeriodCountdownType<u8>;

#[cfg(not(feature = "alloc"))]
pub trait SnoozePeriodCountdown:
Copy + Clone + Send + Sync + QuantizableUInt + 'static
{

}

#[cfg(feature = "alloc")]
pub trait SnoozePeriodCountdown:
Copy + Clone + Send + Sync + Debug + Display + Default + QuantizableUInt + 'static
{

}

impl SnoozePeriodCountdown for u32 {

}

impl SnoozePeriodCountdown for u64 {

}

impl SnoozePeriodCountdown for u16 {

}

impl SnoozePeriodCountdown for u8 {

}

//endregion

//region Snooze Period Limit

pub type SnoozePeriodLimitType<T> = T;
pub type SnoozePeriodLimitU64 = SnoozePeriodLimitType<u64>;
pub type SnoozePeriodLimitU32 = SnoozePeriodLimitType<u32>;
pub type SnoozePeriodLimitU16 = SnoozePeriodLimitType<u16>;
pub type SnoozePeriodLimitU8 = SnoozePeriodLimitType<u8>;

#[cfg(not(feature = "alloc"))]
pub trait SnoozePeriodLimit:
Copy + Clone + Send + Sync + QuantizableUInt + 'static
{

}

#[cfg(feature = "alloc")]
pub trait SnoozePeriodLimit:
Copy + Clone + Send + Sync + Debug + Display + Default + QuantizableUInt + 'static
{

}

impl SnoozePeriodLimit for u64 {

}

impl SnoozePeriodLimit for u32 {

}

impl SnoozePeriodLimit for u16 {

}

impl SnoozePeriodLimit for u8 {

}

//endregion

