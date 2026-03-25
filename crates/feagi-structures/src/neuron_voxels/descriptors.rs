use core::fmt::{Debug, Display};
use crate::base_quantizable::unsigned_integer::QuantizableUInt;
use crate::base_quantizable::value::{QuantizableValue};

/// There is no reason for this to be quantized ever. Defines the number of neurons that a single
/// voxel represents. In most contexts this will be 1, but sometimes may be more.
pub type NumberNeuronsPerVoxel = u8;

/// Data such as neuron voltage potential
//region Potential Unit
crate::define_quantizable_value_type_family!(NeuronPotentialUnit);

//endregion

//region Neuron Voxel Coordinate

crate::define_unsigned_coordinate_3d_type_family!(NeuronVoxelCoordinate);

//endregion

//region Neuron Voxel Dimensions

crate::define_dimension_3d_type_family!(NeuronVoxelDimensions, NeuronVoxelCoordinate);


//endregion


