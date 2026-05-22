/// Stores neuron voxels in a sparse way with a simple linear neuron index, making it generally
/// quite space efficient for cortical areas that are not fully lit.

mod neuron_voxel_ip;
mod neuron_voxel_index_vector;

pub use neuron_voxel_ip::NeuronVoxelIP;
pub use neuron_voxel_index_vector::NeuronVoxelIndexVector;