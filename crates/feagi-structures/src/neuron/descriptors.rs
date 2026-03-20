use core::fmt::{Debug, Display};
use crate::base_quantizable::coordinate::UnsignedCoordinate3DType;
pub use crate::base_quantizable::unsigned_integer::QuantizableUInt;
pub use crate::base_quantizable::value::QuantizableValue;

pub type NumberNeuronsPerVoxel = u8;

//region Potential Unit
crate::define_quantizable_value_type_family!(PotentialUnit);

#[cfg(not(feature = "alloc"))]
pub trait PotentialUnit:
Copy + Clone + Send + Sync + QuantizableValue + 'static
{
}

#[cfg(feature = "alloc")]
pub trait PotentialUnit:
Copy + Clone + Send + Sync + Debug + Display + Default + QuantizableValue + 'static
{
}

#[cfg(feature = "support_64bit_indexing_quantization")]
impl PotentialUnit for PotentialUnitF64 {}
impl PotentialUnit for PotentialUnitF32 {}
impl PotentialUnit for PotentialUnitF16 {}
impl PotentialUnit for PotentialUnitU8 {}
//endregion

//region Neuron Voxel Coordinate
pub type NeuronVoxelCoordinate<T> = UnsignedCoordinate3DType<T>
where
    T: QuantizableUInt;
#[cfg(feature = "support_64bit_indexing_quantization")]
pub type NeuronVoxelCoordinateU64 = NeuronVoxelCoordinate<u64>;
pub type NeuronVoxelCoordinateU32 = NeuronVoxelCoordinate<u32>;
pub type NeuronVoxelCoordinateU16 = NeuronVoxelCoordinate<u16>;
pub type NeuronVoxelCoordinateU8 = NeuronVoxelCoordinate<u8>;
//endregion

//region Neuron Voxel Dimensions
pub type NeuronVoxelDimensions<T> = UnsignedCoordinate3DType<T>
where
    T: QuantizableUInt;
#[cfg(feature = "support_64bit_indexing_quantization")]
pub type NeuronVoxelDimensionsU64 = NeuronVoxelDimensions<u64>;
pub type NeuronVoxelDimensionsU32 = NeuronVoxelDimensions<u32>;
pub type NeuronVoxelDimensionsU16 = NeuronVoxelDimensions<u16>;
pub type NeuronVoxelDimensionsU8 = NeuronVoxelDimensions<u8>;

pub trait NeuronVoxelDimensionsExt<T: QuantizableUInt> {
    fn get_number_neurons(&self, density: NumberNeuronsPerVoxel) -> usize;
}

impl<T: QuantizableUInt> NeuronVoxelDimensionsExt<T> for NeuronVoxelDimensions<T> {
    #[inline(always)]
    fn get_number_neurons(&self, density: NumberNeuronsPerVoxel) -> usize {
        self.x.to_usize() * self.y.to_usize() * self.z.to_usize() * density as usize
    }
}
//endregion


