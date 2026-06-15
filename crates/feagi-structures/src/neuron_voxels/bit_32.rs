//! 32 bit is the "universal" quantization

use feagi_data::quantization_levels::feagi_global_quantization::FeagiGlobalQuantizationStandard;
use feagi_data::quantization_levels::cortical_potential_quantization::CorticalPotentialQuantizationFloat32;
use crate::neuron_voxels::collections::{NeuronVoxelCollectionContiguousVectorGeneric, NeuronVoxelCollectionSparseHashmapGeneric};
use crate::neuron_voxels::voxel_collection_generic_descriptors::*;


pub type NeuronVoxelPotential = NeuronVoxelPotentialGeneric<
    CorticalPotentialQuantizationFloat32
>;
pub type NeuronVoxelAxis = NeuronVoxelAxisGeneric<
    FeagiGlobalQuantizationStandard
>;
pub type NeuronVoxelLinearIndex = NeuronVoxelLinearIndexGeneric<
    FeagiGlobalQuantizationStandard
>;
pub type NeuronVoxelCoordinate = NeuronVoxelCoordinateGeneric<
    FeagiGlobalQuantizationStandard
>;
pub type NeuronVoxelDimensions = NeuronVoxelDimensionsGeneric<
    FeagiGlobalQuantizationStandard
>;

pub type NeuronVoxelContiguousVector = NeuronVoxelCollectionContiguousVectorGeneric<
    FeagiGlobalQuantizationStandard, CorticalPotentialQuantizationFloat32
>;
pub type NeuronVoxelSparseHashmap = NeuronVoxelCollectionSparseHashmapGeneric<
    FeagiGlobalQuantizationStandard, CorticalPotentialQuantizationFloat32
>;