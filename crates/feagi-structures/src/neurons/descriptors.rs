/// The number of neurons that a single voxel represents. In most contexts this will be 1,
/// but sometimes may be more, though never high, hence being locked to a u8
pub type NumberNeuronsPerVoxel = u8;

/// Neuron Potential of neurons (not voxels!)
//region Neuron Membrane Potential

crate::define_quantizable_value_type_family!(NeuronMembranePotential);

//endregion

//region Neuron Count (not voxels!)

crate::define_quantizable_uint_type_family!(NeuronIndex);

//endregion

//region Neuron Count (not voxels!)

crate::define_quantizable_uint_type_family!(NeuronCount);

//endregion