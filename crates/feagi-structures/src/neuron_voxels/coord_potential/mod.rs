/// Coordinate Potential structures store neuron voxels each with the xyz coordinates. Its
/// quite space inefficient but there are some contexts where it makes sense compute wise

mod neuron_voxel_xyzp;

#[cfg(feature = "alloc")]
mod neuron_voxel_coord_vector;

pub use neuron_voxel_xyzp::NeuronVoxelXYZP;
#[cfg(feature = "alloc")]
pub use neuron_voxel_coord_vector::NeuronVoxelCoordVector;

