/// The number of neurons that a single voxel represents. In most contexts this will be 1,
/// but sometimes may be more, though never high, hence being locked to a u8
pub type NumberNeuronsPerVoxel = u8;

/// Neuron Potential of neurons (not voxels!)
//region Potential Unit

crate::define_quantizable_value_type_family!(NeuronPotential);

//endregion
