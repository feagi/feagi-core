use core::fmt::{Debug, Display};
use crate::base_quantizable::coordinate::UnsignedCoordinate3DType;
use crate::base_quantizable::unsigned_integer::QuantizableUInt;
use crate::base_quantizable::value::QuantizableValue;

pub type NeuronVoxelCoordinate<T: QuantizableUInt> = UnsignedCoordinate3DType<T>;

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
pub trait NeuralMembranePotential:
Copy + Clone + Send + Sync + Debug + Display + Default + QuantizableValue + 'static
{
    fn from_f32(value: f32) -> Self;
    fn to_f32(self) -> f32;
    fn saturating_add(self, other: Self) -> Self;
    fn mul_leak(self, leak_coefficient: f32) -> Self;
}


impl NeuralMembranePotential for f32 {

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
