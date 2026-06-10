//! 32 bit is the "universal" quantization

use feagi_data::shared_quantization_sets::{NeuronModelQuantization, FeagiGlobalQuantization, CorticalPotentialQuantizationFloat32, FeagiGlobalQuantizationStandard};
use crate::neuron_voxels::collections::{NeuronVoxelCollectionContiguousVectorGeneric, NeuronVoxelCollectionSparseHashmapGeneric};
use crate::neuron_voxels::voxel_collection_generic_descriptors::*;


pub type NeuronVoxelPotential = NeuronVoxelPotentialGeneric<
    CorticalPotentialQuantizationFloat32
>;
pub type NeuronVoxelAxis = NeuronVoxelAxisGeneric<
    CorticalPotentialQuantizationFloat32
>;
pub type NeuronVoxelLinearIndex = NeuronVoxelLinearIndexGeneric<
    CorticalPotentialQuantizationFloat32
>;
pub type NeuronVoxelCoordinate = NeuronVoxelCoordinateGeneric<
    CorticalPotentialQuantizationFloat32
>;
pub type NeuronVoxelDimensions = NeuronVoxelDimensionsGeneric<
    CorticalPotentialQuantizationFloat32
>;

pub type NeuronVoxelContiguousVector = NeuronVoxelCollectionContiguousVectorGeneric<
    FeagiGlobalQuantizationStandard, CorticalPotentialQuantizationFloat32
>;
pub type NeuronVoxelSparseHashmap = NeuronVoxelCollectionSparseHashmapGeneric<
    FeagiGlobalQuantizationStandard, CorticalPotentialQuantizationFloat32
>;