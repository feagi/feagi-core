use core::fmt::{Debug, Display};

/// The number of neurons that are in a voxel. Realistically this will always be 1 or close to it.
pub type NumberNeuronsPerVoxel = u8;

//region Neuron ID
pub type InterneuronIDType<T: QuantizableUInt> = T;
#[cfg(feature = "support_64bit_indexing_quantization")]
pub type InterneuronIDU64 = InterneuronIDType<u64>;
pub type InterneuronIDU32 = InterneuronIDType<u32>;
pub type InterneuronIDU16 = InterneuronIDType<u16>;
pub type InterneuronIDU8 = InterneuronIDType<u8>;

#[cfg(not(feature = "alloc"))]
pub trait InterneuronID:
Copy + Clone + Send + Sync + QuantizableUInt + 'static
{

}

#[cfg(feature = "alloc")]
pub trait InterneuronNeuronId:
Copy + Clone + Send + Sync + Debug + Display + Default + QuantizableUInt + 'static
{

}

#[cfg(feature = "support_64bit_indexing_quantization")]
impl InterneuronID for u64 {

}

impl InterneuronID for u32 {

}

impl InterneuronID for u16 {

}

impl InterneuronID for u8 {

}

//endregion

//region Leak Coefficient

pub type LeakCoefficientType<T: QuantizableValue> = T;
#[cfg(feature = "support_64bit_indexing_quantization")]
pub type LeakCoefficientU64 = LeakCoefficientType<f64>;
pub type LeakCoefficientU32 = LeakCoefficientType<f32>;
pub type LeakCoefficientU16 = LeakCoefficientType<f16>;
pub type LeakCoefficientU8 = LeakCoefficientType<u8>;


#[cfg(not(feature = "alloc"))]
pub trait LeakCoefficient:
Copy + Clone + Send + Sync + QuantizableValue + 'static
{
    fn saturating_add(self, other: Self) -> Self;
    fn mul_leak(self, leak_coefficient: f32) -> Self;
}

#[cfg(feature = "alloc")]
pub trait LeakCoefficient:
Copy + Clone + Send + Sync + Debug + Display + Default + QuantizableValue + 'static
{
    fn saturating_add(self, other: Self) -> Self;
    fn mul_leak(self, leak_coefficient: f32) -> Self;
}

#[cfg(feature = "support_64bit_indexing_quantization")]
impl LeakCoefficient for f64 {

}

impl LeakCoefficient for f32 {

}

impl LeakCoefficient for f16 {

}

impl LeakCoefficient for u8 {

}

//endregion

//region Refractory Period

pub type RefractoryPeriodType<T: QuantizableUInt> = T;
#[cfg(feature = "support_64bit_indexing_quantization")]
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

#[cfg(feature = "support_64bit_indexing_quantization")]
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

pub type RefractoryCountdownType<T: QuantizableUInt> = T;
#[cfg(feature = "support_64bit_indexing_quantization")]
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

#[cfg(feature = "support_64bit_indexing_quantization")]
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

pub type ExcitabilityType<T: QuantizableValue> = T;
#[cfg(feature = "support_64bit_indexing_quantization")]
pub type ExcitabilityU64 = ExcitabilityType<f64>;
pub type ExcitabilityU32 = ExcitabilityType<f32>;
pub type ExcitabilityU16 = ExcitabilityType<f16>;
pub type ExcitabilityU8 = ExcitabilityType<u8>;


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

pub type ConsecutiveFireCountdownType<T: QuantizableUInt> = T;
#[cfg(feature = "support_64bit_indexing_quantization")]
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

#[cfg(feature = "support_64bit_indexing_quantization")]
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

pub type ConsecutiveFireLimitType<T: QuantizableUInt> = T;
#[cfg(feature = "support_64bit_indexing_quantization")]
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

#[cfg(feature = "support_64bit_indexing_quantization")]
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

pub type SnoozePeriodCountdownType<T: QuantizableUInt> = T;
#[cfg(feature = "support_64bit_indexing_quantization")]
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

#[cfg(feature = "support_64bit_indexing_quantization")]
impl SnoozePeriodCountdown for u64 {

}

impl SnoozePeriodCountdown for u32 {

}

impl SnoozePeriodCountdown for u16 {

}

impl SnoozePeriodCountdown for u8 {

}

//endregion

//region Snooze Period Limit

pub type SnoozePeriodLimitType<T: QuantizableUInt> = T;
#[cfg(feature = "support_64bit_indexing_quantization")]
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

#[cfg(feature = "support_64bit_indexing_quantization")]
impl SnoozePeriodLimit for u64 {

}

impl SnoozePeriodLimit for u32 {

}

impl SnoozePeriodLimit for u16 {

}

impl SnoozePeriodLimit for u8 {

}

//endregion