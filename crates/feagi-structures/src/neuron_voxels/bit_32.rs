//! 32 bit is the "universal" quantization

use feagi_data::shared_quantization_sets::CorticalAreaModelQuantization;
use crate::neuron_voxels::nondense_collections::{NeuronVoxelCollectionContiguousVectorGeneric, NeuronVoxelCollectionSparseHashmapGeneric};
use crate::neuron_voxels::voxel_collection_generic_descriptors::*;

pub struct CANQ32;
impl CorticalAreaModelQuantization for CANQ32 {
    type GlobalBurstIndexQuant = u32; // we never use this here
    type NeuronIndexCountQuant = u32;
    type NeuronPotentialQuant = f32;
}

pub type NeuronVoxelPotential = NeuronVoxelPotentialGeneric<f32>;
pub type NeuronVoxelAxis = NeuronVoxelAxisGeneric<u32>;
pub type NeuronVoxelLinearIndex = NeuronVoxelLinearIndexGeneric<u32>;
pub type NeuronVoxelCoordinate = NeuronVoxelCoordinateGeneric<u32>;
pub type NeuronVoxelDimensions = NeuronVoxelDimensionsGeneric<u32>;

pub type NeuronVoxelContiguousVector = NeuronVoxelCollectionContiguousVectorGeneric<CANQ32>;
pub type NeuronVoxelSparseHashmap = NeuronVoxelCollectionSparseHashmapGeneric<CANQ32>;