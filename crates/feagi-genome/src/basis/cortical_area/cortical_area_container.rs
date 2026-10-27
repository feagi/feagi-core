use std::collections::HashSet;
use feagi_brain_basis::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::identifiers::brain_regions::BrainRegionID;
use crate::basis::cortical_area::cortical_id::CorticalID;
use crate::basis::genome_object_metadata::GenomeObjectMetadata;

// TODO premade types

pub struct CorticalAreaContainer<FIQ, CacheData>
where
    FIQ: FeagiIndexQuantization,
    CacheData: CorticalCacheDataType,

// TODO neuron model as a typed enum for model with deeper fields
{
    pub metadata: GenomeObjectMetadata,
    pub model_data: (), // TODO Model Data Enum, Takes generic for async / sync for connectome data
    pub cache: CacheData,
    

    _p: core::marker::PhantomData<FIQ>
}

//region Cache Data

pub trait CorticalCacheDataType: Sized {
    fn build_cache(
        parent_region: BrainRegionID,
        incoming_mappings: HashSet<CorticalID>,
        outgoing_mappings: HashSet<CorticalID>
    ) -> Self;

    // TODO funcs
}

/// Don't cache anything
impl CorticalCacheDataType for () {
    fn build_cache(
        parent_region: BrainRegionID,
        incoming_mappings: HashSet<CorticalID>,
        outgoing_mappings: HashSet<CorticalID>
    ) -> Self {
        () // nothing!
    }

    // TODO funcs
}

/// Caches some data that is technically not needed but good for fast access. Also has async accessors
/// for getting data from the NPU
pub struct CorticalAreaRuntimeCache {
    pub parent_region: BrainRegionID,
    pub incoming_mappings: HashSet<CorticalID>,
    pub outgoing_mappings: HashSet<CorticalID>,
}

impl CorticalCacheDataType for CorticalAreaRuntimeCache {
    fn build_cache(
        parent_region: BrainRegionID,
        incoming_mappings: HashSet<CorticalID>,
        outgoing_mappings: HashSet<CorticalID>
    ) -> Self {
        Self {
            parent_region,
            incoming_mappings,
            outgoing_mappings,
        }
    }
}

//endregion

//region Cortical Area Data Lookups

/// Describes how to get some data
pub trait CorticalAreaDataLookups {}

/// No additional data caching, useful for storage / export
pub struct CorticalAreaStoredDataLookup;
impl CorticalAreaDataLookups for CorticalAreaStoredDataLookup {}
impl CorticalAreaStoredDataLookup {
    // TODO Sync Accessors
}

//endregion



