//! 32 bit is the "universal" quantization

use feagi_data::shared_quantization_sets::{NeuronModelQuantization, FeagiGlobalQuantization};
use crate::neuron_voxels::nondense_collections::{NeuronVoxelCollectionContiguousVectorGeneric, NeuronVoxelCollectionSparseHashmapGeneric};
use crate::neuron_voxels::voxel_collection_generic_descriptors::*;

#[doc(hidden)]
pub struct CAIQ32;

impl FeagiGlobalQuantization for CAIQ32 {
    type GlobalBurstIndexQuant = u32;
    type NeuronIndexCountQuant = u32;
    
    // These ones aren't used here, but whatever
    type SynapseIndexCountQuant = u32;
    type CorticalAreaIndexCountQuant = u32;
    type AxonBundleIndexCountQuant = u32;
}

#[doc(hidden)]
pub struct CANQ32;

impl NeuronModelQuantization for CANQ32 {
    type NeuronPotentialQuant = f32;
}

pub type NeuronVoxelPotential = NeuronVoxelPotentialGeneric<<CANQ32 as NeuronModelQuantization>::NeuronPotentialQuant>;
pub type NeuronVoxelAxis = NeuronVoxelAxisGeneric<<CAIQ32 as FeagiGlobalQuantization>::NeuronIndexCountQuant>;
pub type NeuronVoxelLinearIndex = NeuronVoxelLinearIndexGeneric<<CAIQ32 as FeagiGlobalQuantization>::NeuronIndexCountQuant>;
pub type NeuronVoxelCoordinate = NeuronVoxelCoordinateGeneric<<CAIQ32 as FeagiGlobalQuantization>::NeuronIndexCountQuant>;
pub type NeuronVoxelDimensions = NeuronVoxelDimensionsGeneric<<CAIQ32 as FeagiGlobalQuantization>::NeuronIndexCountQuant>;

pub type NeuronVoxelContiguousVector = NeuronVoxelCollectionContiguousVectorGeneric<CAIQ32, CANQ32>;
pub type NeuronVoxelSparseHashmap = NeuronVoxelCollectionSparseHashmapGeneric<CAIQ32, CANQ32>;