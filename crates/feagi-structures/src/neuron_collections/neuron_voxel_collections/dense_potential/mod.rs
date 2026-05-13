/// Dense neuron voxel collections are not sparse at all, but rather store a potential for every
/// voxel in a cortical area. Since they don't change size with more neuron_collections, they are the only
/// option for embedded contexts and are very performant. They are also more space efficient
/// than other collection types with a cortical area is mostly lit anyways.

// NOTE: No need for a neuron struct here as the literal only value would be the voxel potential itself

mod neuron_voxel_dense_array;

#[cfg(feature = "alloc")]
mod neuron_voxel_dense_vector;

pub use neuron_voxel_dense_array::NeuronVoxelDenseArray;

#[cfg(feature = "alloc")]
pub use neuron_voxel_dense_vector::NeuronVoxelDenseVector;

#[cfg(feature = "alloc")]
pub use crate::cortical_area__neuron_data_collections::multi_neuron_voxel_dense_vector::MultiNeuronVoxelDenseVector;