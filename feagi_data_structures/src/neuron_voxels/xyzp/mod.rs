/// XYZP is a format where each neuron voxel stores its x y z position, and the voxels potential as float p. Fast and easy, but not memory efficient.

mod neuron_voxel_xyzp;
mod neuron_voxel_xyzp_arrays;
mod cortical_mapped_xyzp_neuron_voxels;

pub use neuron_voxel_xyzp::NeuronVoxelXYZP;
pub use neuron_voxel_xyzp_arrays::NeuronVoxelXYZPArrays;
pub use cortical_mapped_xyzp_neuron_voxels::CorticalMappedXYZPNeuronVoxels;