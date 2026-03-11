use core::fmt::{Debug, Display};
use crate::base_quantizable::quantizable_floats::QuantizableFloat;
use crate::base_quantizable::quantizable_uints::QuantizableUInt;
use crate::common_descriptors::{Coordinate3D, Dimension3D};

pub type NeuronVoxelCoordinate = Coordinate3D;

//region Neuron ID
pub type NeuronIdU32 = u32;
pub type NeuronIdU16 = u16;
pub type NeuronIdU8 = u8;

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

impl NeuronId for u32 {

}

impl NeuronId for u16 {

}

impl NeuronId for u8 {

}

//endregion

//region Neuron Count
pub type NeuronCountU32 = u32;
pub type NeuronCountU16 = u16;
pub type NeuronCountU8 = u8;

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
Copy + Clone + Send + Sync + QuantizableFloat + 'static
{
    fn from_f32(value: f32) -> Self;
    fn to_f32(self) -> f32;
    fn saturating_add(self, other: Self) -> Self;
    fn mul_leak(self, leak_coefficient: f32) -> Self;
}

#[cfg(feature = "alloc")]
pub trait NeuralPotentialValue:
Copy + Clone + Send + Sync + Debug + Display + Default + QuantizableFloat + 'static
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
Copy + Clone + Send + Sync + QuantizableFloat + 'static
{
    fn from_f32(value: f32) -> Self;
    fn to_f32(self) -> f32;
    fn saturating_add(self, other: Self) -> Self;
    fn mul_leak(self, leak_coefficient: f32) -> Self;
}

#[cfg(feature = "alloc")]
pub trait LeakCoefficient:
Copy + Clone + Send + Sync + Debug + Display + Default + QuantizableFloat + 'static
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

pub type RefractoryPeriodU32 = u32;
pub type RefractoryPeriodU16 = u16;
pub type RefractoryPeriodU8 = u8;

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

impl RefractoryPeriod for u32 {

}

impl RefractoryPeriod for u16 {

}

impl RefractoryPeriod for u8 {

}

//endregion

//region Refractory Countdown

pub type RefractoryCountdownU32 = u32;
pub type RefractoryCountdownU16 = u16;
pub type RefractoryCountdownU8 = u8;

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
Copy + Clone + Send + Sync + QuantizableFloat + 'static
{
    fn from_f32(value: f32) -> Self;
    fn to_f32(self) -> f32;
    fn saturating_add(self, other: Self) -> Self;
    fn mul_leak(self, leak_coefficient: f32) -> Self;
}

#[cfg(feature = "alloc")]
pub trait Excitability:
Copy + Clone + Send + Sync + Debug + Display + Default + QuantizableFloat + 'static
{

}


impl Excitability for f32 {
    
}

//endregion

//region Consecutive Fire Countdown

pub type ConsecutiveFireCountdownU32 = u32;
pub type ConsecutiveFireCountdownU16 = u16;
pub type ConsecutiveFireCountdownU8 = u8;

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

impl ConsecutiveFireCountdown for u32 {

}

impl ConsecutiveFireCountdown for u16 {

}

impl ConsecutiveFireCountdown for u8 {

}

//endregion

//region Consecutive Fire Limit

pub type ConsecutiveFireLimitU32 = u32;
pub type ConsecutiveFireLimitU16 = u16;
pub type ConsecutiveFireLimitU8 = u8;

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

impl ConsecutiveFireLimit for u32 {

}

impl ConsecutiveFireLimit for u16 {

}

impl ConsecutiveFireLimit for u8 {

}

//endregion

//region Snooze Period Countdown

pub type SnoozePeriodCountdownU32 = u32;
pub type SnoozePeriodCountdownU16 = u16;
pub type SnoozePeriodCountdownU8 = u8;

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

impl SnoozePeriodCountdown for u16 {

}

impl SnoozePeriodCountdown for u8 {

}

//endregion

//region Snooze Period Limit

pub type SnoozePeriodLimitU32 = u32;
pub type SnoozePeriodLimitU16 = u16;
pub type SnoozePeriodLimitU8 = u8;

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

impl SnoozePeriodLimit for u32 {

}

impl SnoozePeriodLimit for u16 {

}

impl SnoozePeriodLimit for u8 {

}

//endregion

