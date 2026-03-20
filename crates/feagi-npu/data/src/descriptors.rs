use core::fmt::{Debug, Display};
use half::f16;
use crate::base_quantizable::coordinate::UnsignedCoordinate3DType;
use crate::base_quantizable::unsigned_integer::QuantizableUInt;
use crate::base_quantizable::value::QuantizableValue;

// TODO macros to core crate

/// Defines all implementations of QuantizableUInt and its dependent traits
macro_rules! impl_quantizable_uint_wrapper {
    ($wrapper:ident) => {

        impl<T: QuantizableUInt> From<T> for $wrapper<T> {
            #[inline(always)]
            fn from(value: T) -> Self {
                Self(value)
            }
        }

        #[cfg(feature = "alloc")]
        impl<T: QuantizableUInt> Default for $wrapper<T> {
            #[inline(always)]
            fn default() -> Self {
                Self(T::default())
            }
        }

        impl<T: QuantizableUInt> From<$wrapper<T>> for usize {
            #[inline(always)]
            fn from(value: $wrapper<T>) -> Self {
                value.0.to_usize()
            }
        }

        impl<T: QuantizableUInt> core::ops::Add for $wrapper<T> {
            type Output = Self;
            #[inline(always)]
            fn add(self, rhs: Self) -> Self::Output {
                Self(self.0 + rhs.0)
            }
        }

        impl<T: QuantizableUInt> core::ops::Sub for $wrapper<T> {
            type Output = Self;
            #[inline(always)]
            fn sub(self, rhs: Self) -> Self::Output {
                Self(self.0 - rhs.0)
            }
        }

        impl<T: QuantizableUInt> core::ops::Mul for $wrapper<T> {
            type Output = Self;
            #[inline(always)]
            fn mul(self, rhs: Self) -> Self::Output {
                Self(self.0 * rhs.0)
            }
        }

        impl<T: QuantizableUInt> core::ops::Div for $wrapper<T> {
            type Output = Self;
            #[inline(always)]
            fn div(self, rhs: Self) -> Self::Output {
                Self(self.0 / rhs.0)
            }
        }

        impl<T: QuantizableUInt> core::ops::AddAssign for $wrapper<T> {
            #[inline(always)]
            fn add_assign(&mut self, rhs: Self) {
                self.0 += rhs.0;
            }
        }

        impl<T: QuantizableUInt> core::ops::SubAssign for $wrapper<T> {
            #[inline(always)]
            fn sub_assign(&mut self, rhs: Self) {
                self.0 -= rhs.0;
            }
        }

        impl<T: QuantizableUInt> core::ops::MulAssign for $wrapper<T> {
            #[inline(always)]
            fn mul_assign(&mut self, rhs: Self) {
                self.0 *= rhs.0;
            }
        }

        impl<T: QuantizableUInt> core::ops::DivAssign for $wrapper<T> {
            #[inline(always)]
            fn div_assign(&mut self, rhs: Self) {
                self.0 /= rhs.0;
            }
        }

        #[cfg(feature = "alloc")]
        impl<T: QuantizableUInt + core::fmt::Display> core::fmt::Display for $wrapper<T> {
            #[inline(always)]
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl<T: QuantizableUInt> QuantizableUInt for $wrapper<T> {
            const NUMBER_OF_BYTES: usize = T::NUMBER_OF_BYTES;
            const ZERO: Self = Self(T::ZERO);
            const ONE: Self = Self(T::ONE);
            const MAX_VALUE: Self = Self(T::MAX_VALUE);
            const MIN_VALUE: Self = Self(T::MIN_VALUE);

            #[inline(always)]
            fn saturating_add(self, other: Self) -> Self {
                Self(self.0.saturating_add(other.0))
            }

            #[inline(always)]
            fn checked_add(self, other: Self) -> Option<Self> {
                self.0.checked_add(other.0).map(Self)
            }

            #[inline(always)]
            fn saturating_sub(self, other: Self) -> Self {
                Self(self.0.saturating_sub(other.0))
            }

            #[inline(always)]
            fn checked_sub(self, other: Self) -> Option<Self> {
                self.0.checked_sub(other.0).map(Self)
            }

            #[inline(always)]
            fn saturating_mul(self, other: Self) -> Self {
                Self(self.0.saturating_mul(other.0))
            }

            #[inline(always)]
            fn checked_mul(self, other: Self) -> Option<Self> {
                self.0.checked_mul(other.0).map(Self)
            }

            #[inline(always)]
            fn checked_div(self, other: Self) -> Option<Self> {
                self.0.checked_div(other.0).map(Self)
            }

            #[inline(always)]
            fn to_usize(self) -> usize {
                self.0.to_usize()
            }

            #[inline(always)]
            fn from_usize(value: usize) -> Self {
                Self(T::from_usize(value))
            }
        }
    };
}

/// Defines a transparent wrapper type and all quantized-width aliases.
macro_rules! define_quantizable_uint_type_family {
    ($base_name:ident) => {
        paste::paste! {
            #[repr(transparent)]
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
            pub struct [<$base_name Type>]<T: QuantizableUInt>(pub T);

            impl_quantizable_uint_wrapper!([<$base_name Type>]);

            #[cfg(feature = "support_64bit_indexing_quantization")]
            pub type [<$base_name U64>] = [<$base_name Type>]<u64>;
            pub type [<$base_name U32>] = [<$base_name Type>]<u32>;
            pub type [<$base_name U16>] = [<$base_name Type>]<u16>;
            pub type [<$base_name U8>] = [<$base_name Type>]<u8>;
        }
    };
}

/// Defines all implementations of QuantizableValue and its dependent traits.
macro_rules! impl_quantizable_value_wrapper {
    ($wrapper:ident) => {
        impl<T: QuantizableValue> From<T> for $wrapper<T> {
            #[inline(always)]
            fn from(value: T) -> Self {
                Self(value)
            }
        }

        #[cfg(feature = "alloc")]
        impl<T: QuantizableValue> Default for $wrapper<T> {
            #[inline(always)]
            fn default() -> Self {
                Self(T::default())
            }
        }

        impl<T: QuantizableValue> From<$wrapper<T>> for f32 {
            #[inline(always)]
            fn from(value: $wrapper<T>) -> Self {
                value.0.to_f32()
            }
        }

        impl<T: QuantizableValue> core::ops::Add for $wrapper<T> {
            type Output = Self;
            #[inline(always)]
            fn add(self, rhs: Self) -> Self::Output {
                Self(self.0 + rhs.0)
            }
        }

        impl<T: QuantizableValue> core::ops::Sub for $wrapper<T> {
            type Output = Self;
            #[inline(always)]
            fn sub(self, rhs: Self) -> Self::Output {
                Self(self.0 - rhs.0)
            }
        }

        impl<T: QuantizableValue> core::ops::Mul for $wrapper<T> {
            type Output = Self;
            #[inline(always)]
            fn mul(self, rhs: Self) -> Self::Output {
                Self(self.0 * rhs.0)
            }
        }

        impl<T: QuantizableValue> core::ops::Div for $wrapper<T> {
            type Output = Self;
            #[inline(always)]
            fn div(self, rhs: Self) -> Self::Output {
                Self(self.0 / rhs.0)
            }
        }

        impl<T: QuantizableValue> core::ops::AddAssign for $wrapper<T> {
            #[inline(always)]
            fn add_assign(&mut self, rhs: Self) {
                self.0 += rhs.0;
            }
        }

        impl<T: QuantizableValue> core::ops::SubAssign for $wrapper<T> {
            #[inline(always)]
            fn sub_assign(&mut self, rhs: Self) {
                self.0 -= rhs.0;
            }
        }

        impl<T: QuantizableValue> core::ops::MulAssign for $wrapper<T> {
            #[inline(always)]
            fn mul_assign(&mut self, rhs: Self) {
                self.0 *= rhs.0;
            }
        }

        impl<T: QuantizableValue> core::ops::DivAssign for $wrapper<T> {
            #[inline(always)]
            fn div_assign(&mut self, rhs: Self) {
                self.0 /= rhs.0;
            }
        }

        #[cfg(feature = "alloc")]
        impl<T: QuantizableValue + core::fmt::Display> core::fmt::Display for $wrapper<T> {
            #[inline(always)]
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl<T: QuantizableValue> QuantizableValue for $wrapper<T> {
            const NUMBER_OF_BYTES: usize = T::NUMBER_OF_BYTES;
            const ZERO: Self = Self(T::ZERO);
            const ONE: Self = Self(T::ONE);
            const MAX_VALUE: Self = Self(T::MAX_VALUE);
            const MIN_VALUE: Self = Self(T::MIN_VALUE);

            #[inline(always)]
            fn saturating_add(self, other: Self) -> Self {
                Self(self.0.saturating_add(other.0))
            }

            #[inline(always)]
            fn checked_add(self, other: Self) -> Option<Self> {
                self.0.checked_add(other.0).map(Self)
            }

            #[inline(always)]
            fn saturating_sub(self, other: Self) -> Self {
                Self(self.0.saturating_sub(other.0))
            }

            #[inline(always)]
            fn checked_sub(self, other: Self) -> Option<Self> {
                self.0.checked_sub(other.0).map(Self)
            }

            #[inline(always)]
            fn saturating_mul(self, other: Self) -> Self {
                Self(self.0.saturating_mul(other.0))
            }

            #[inline(always)]
            fn checked_mul(self, other: Self) -> Option<Self> {
                self.0.checked_mul(other.0).map(Self)
            }

            #[inline(always)]
            fn checked_div(self, other: Self) -> Option<Self> {
                self.0.checked_div(other.0).map(Self)
            }

            #[inline(always)]
            fn to_f32(self) -> f32 {
                self.0.to_f32()
            }

            #[inline(always)]
            fn from_f32(value: f32) -> Self {
                Self(T::from_f32(value))
            }
        }
    };
}

/// Defines a transparent value wrapper type and all quantized-value aliases.
macro_rules! define_quantizable_value_type_family {
    ($base_name:ident) => {
        paste::paste! {
            #[repr(transparent)]
            #[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
            pub struct [<$base_name Type>]<T: QuantizableValue>(pub T);

            impl_quantizable_value_wrapper!([<$base_name Type>]);

            #[cfg(feature = "support_64bit_indexing_quantization")]
            pub type [<$base_name F64>] = [<$base_name Type>]<f64>;
            pub type [<$base_name F32>] = [<$base_name Type>]<f32>;
            pub type [<$base_name F16>] = [<$base_name Type>]<f16>;
            pub type [<$base_name U8>] = [<$base_name Type>]<u8>;
        }
    };
}



/// The number of neurons that are in a voxel. Realistically this will always be 1 or close to it.
pub type NumberNeuronsPerVoxel = u8;


/// Refers to the count of bursts, which is often used as a measurement of delta time. Note that
/// while the global burst counter can become a large number thus should not be quantized low, the
/// numbers that these units represent may have lower values, so can possibly be quantized lower.
///
/// Can represent Neuron Refractory Period, Neuron Refractory Countdown, Snooze Period Limit,
/// Snooze Period Countdown, Consecutive Fire Limit, Consecutive Fire Countdown
//region Burst Delta Count
define_quantizable_uint_type_family!(BurstDeltaCount);

#[cfg(not(feature = "alloc"))]
pub trait BurstDeltaCount:
Copy + Clone + Send + Sync + QuantizableUInt + 'static
{

}

#[cfg(feature = "alloc")]
pub trait BurstDeltaCount:
Copy + Clone + Send + Sync + Debug + Display + Default + QuantizableUInt + 'static
{

}

#[cfg(feature = "support_64bit_indexing_quantization")]
impl BurstDeltaCount for BurstDeltaCountU64 {}
impl BurstDeltaCount for BurstDeltaCountU32 {}
impl BurstDeltaCount for BurstDeltaCountU16 {}
impl BurstDeltaCount for BurstDeltaCountU8 {}

//endregion


/// Index for interneurons in FEAGI Connectome, where every neuron gets its own unique index.
/// Depending on compile target and memory restrictions, the usage of this may be implemented in
/// various ways.
//region Interneuron Index
define_quantizable_uint_type_family!(InterneuronIndex);

#[cfg(not(feature = "alloc"))]
pub trait InterneuronIndex:
Copy + Clone + Send + Sync + QuantizableUInt + 'static
{

}

#[cfg(feature = "alloc")]
pub trait InterneuronIndex:
Copy + Clone + Send + Sync + Debug + Display + Default + QuantizableUInt + 'static
{

}

#[cfg(feature = "support_64bit_indexing_quantization")]
impl InterneuronIndex for InterneuronIndexIDU64 {}
impl InterneuronIndex for InterneuronIndexIDU32 {}
impl InterneuronIndex for InterneuronIndexIDU16 {}
impl InterneuronIndex for InterneuronIndexIDU8 {}

//endregion



/// Index for Cortical Areas in FEAGI Connectome, where every neuron gets its own unique index.
/// Depending on compile target and memory restrictions, the usage of this may be implemented in
/// various ways. Not to be confused with CorticalIDs, which is a genome level identifier.
//region Cortical Index
define_quantizable_uint_type_family!(CorticalAreaIndex);

#[cfg(not(feature = "alloc"))]
pub trait CorticalAreaIndex:
Copy + Clone + Send + Sync + QuantizableUInt + 'static
{

}

#[cfg(feature = "alloc")]
pub trait CorticalAreaIndex:
Copy + Clone + Send + Sync + Debug + Display + Default + QuantizableUInt + 'static
{

}

#[cfg(feature = "support_64bit_indexing_quantization")]
impl CorticalAreaIndex for CorticalAreaIndexIDU64 {}
impl CorticalAreaIndex for CorticalAreaIndexIDU32 {}
impl CorticalAreaIndex for CorticalAreaIndexIDU16 {}
impl CorticalAreaIndex for CorticalAreaIndexIDU8 {}

//endregion



/// Index for synapses in FEAGI, where every synapse gets its own unique index. Depending on
/// compile target and memory restrictions, the usage of this may be implemented in various ways.
//region Synapse Index
define_quantizable_uint_type_family!(SynapseIndex);

#[cfg(not(feature = "alloc"))]
pub trait SynapseIndex:
Copy + Clone + Send + Sync + QuantizableUInt + 'static
{

}

#[cfg(feature = "alloc")]
pub trait SynapseIndex:
Copy + Clone + Send + Sync + Debug + Display + Default + QuantizableUInt + 'static
{

}

#[cfg(feature = "support_64bit_indexing_quantization")]
impl SynapseIndex for SynapseIndexU64 {}
impl SynapseIndex for SynapseIndexU32 {}
impl SynapseIndex for SynapseIndexU16 {}
impl SynapseIndex for SynapseIndexU8 {}

//endregion


/// Broadly speaking, the measurement of voltage potential, often in the context of membrane
/// dynamics. However, specific implementation and quantization used will impact the actual
/// math used.
///
/// Can represent Neuron Membrane Potential, Firing Threshold, Firing Threshold Limit,
/// Synaptic Weight, PostSynaptic Potential Max, Degeneracy Constant
/// TODO "Synaptic Weight" may need to renamed to something more descriptive
//region Potential Unit
define_quantizable_value_type_family!(PotentialUnit);

#[cfg(not(feature = "alloc"))]
pub trait PotentialUnit:
Copy + Clone + Send + Sync + QuantizableValue + 'static
{

}

#[cfg(feature = "alloc")]
pub trait PotentialUnit:
Copy + Clone + Send + Sync + Debug + Display + Default + QuantizableUInt + 'static
{

}

#[cfg(feature = "support_64bit_indexing_quantization")]
impl PotentialUnit for PotentialUnitU64 {}
impl PotentialUnit for PotentialUnitU32 {}
impl PotentialUnit for PotentialUnitU16 {}
impl PotentialUnit for PotentialUnitU8 {}

//endregion

/// Broadly speaking, represents percentage values from 0% - 100%, however may exceed those bounds
/// in some circumstances, so no enforcing checks are used, Often used to as multipliers.
///
/// Can represent Neuron Excitability, Synaptic Attractivity, Leak Variability, Leak Coefficient
// TODO its possible PotentialUnit is not a good fit for this. We should review
//region PercentageScale
define_quantizable_value_type_family!(PercentageScale);

#[cfg(not(feature = "alloc"))]
pub trait PercentageScale:
Copy + Clone + Send + Sync + QuantizableValue + 'static
{

}

#[cfg(feature = "alloc")]
pub trait PercentageScale:
Copy + Clone + Send + Sync + Debug + Display + Default + QuantizableUInt + 'static
{

}

#[cfg(feature = "support_64bit_indexing_quantization")]
impl PercentageScale for PercentageScaleU64 {}
impl PercentageScale for PercentageScaleU32 {}
impl PercentageScale for PercentageScaleU16 {}
impl PercentageScale for PercentageScaleU8 {}

//endregion


//region Potential Unit
define_quantizable_value_type_family!(PSPMultiplier);

#[cfg(not(feature = "alloc"))]
pub trait PSPMultiplier:
Copy + Clone + Send + Sync + QuantizableValue + 'static
{

}

#[cfg(feature = "alloc")]
pub trait PSPMultiplier:
Copy + Clone + Send + Sync + Debug + Display + Default + QuantizableUInt + 'static
{

}

#[cfg(feature = "support_64bit_indexing_quantization")]
impl PSPMultiplier for PSPMultiplierU64 {}
impl PSPMultiplier for PSPMultiplierU32 {}
impl PSPMultiplier for PSPMultiplierU16 {}
impl PSPMultiplier for PSPMultiplierU8 {}

//endregion

/// Represents what coordinate a voxel (relative to the root of its Cortical Area) is in.
/// Note in some cases 1 voxel may contain multiple neurons
//region Neuron Voxel Coordinate

pub type NeuronVoxelCoordinate<T: QuantizableUInt> = UnsignedCoordinate3DType<T>;
#[cfg(feature = "support_64bit_indexing_quantization")]
pub type NeuronVoxelCoordinateU64 = NeuronVoxelCoordinate<u64>;
pub type NeuronVoxelCoordinateU32 = NeuronVoxelCoordinate<u32>;
pub type NeuronVoxelCoordinateU16 = NeuronVoxelCoordinate<u16>;
pub type NeuronVoxelCoordinateU8 = NeuronVoxelCoordinate<u8>;

//endregion

/// Represents the dimensions (in voxels) of a cortical area
/// Note in some cases 1 voxel may contain multiple neurons
//region Neuron Voxel Dimensions

pub type NeuronVoxelDimensions<T: QuantizableUInt> = UnsignedCoordinate3DType<T>;
#[cfg(feature = "support_64bit_indexing_quantization")]
pub type NeuronVoxelDimensionsU64 = NeuronVoxelDimensions<u64>;
pub type NeuronVoxelDimensionsU32 = NeuronVoxelDimensions<u32>;
pub type NeuronVoxelDimensionsU16 = NeuronVoxelDimensions<u16>;
pub type NeuronVoxelDimensionsU8 = NeuronVoxelDimensions<u8>;

//endregion



