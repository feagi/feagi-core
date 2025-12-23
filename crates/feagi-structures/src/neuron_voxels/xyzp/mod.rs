mod cortical_mapped_xyzp_neuron_voxels;
/// XYZP neuron voxel representation.
///
/// Each voxel stores its (x, y, z) position and activation potential (p) as floats.
/// Simple and fast access, but not very memory efficient
mod neuron_voxel_xyzp;
mod neuron_voxel_xyzp_arrays;

pub use cortical_mapped_xyzp_neuron_voxels::CorticalMappedXYZPNeuronVoxels;
pub use neuron_voxel_xyzp::NeuronVoxelXYZP;
pub use neuron_voxel_xyzp_arrays::NeuronVoxelXYZPArrays;
