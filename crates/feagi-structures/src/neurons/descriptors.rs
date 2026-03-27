/// There is no reason for this to be quantized ever. Defines the number of neurons that a single
/// voxel represents. In most contexts this will be 1, but sometimes may be more.
pub type NumberNeuronsPerVoxel = u8;

/// Neuron Potential of neurons (not voxels!)
//region Potential Unit

crate::define_quantizable_value_type_family!(NeuronPotential);

//endregion
