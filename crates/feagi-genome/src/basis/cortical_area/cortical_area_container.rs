use std::collections::HashSet;
use feagi_brain_basis::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::identifiers::brain_regions::BrainRegionID;
use crate::basis::cortical_area::cortical_id::CorticalID;
use crate::basis::genome_object_metadata::GenomeObjectMetadata;

// TODO premade types

pub struct CorticalAreaContainer<FIQ, CachingStrategy>
where
    FIQ: FeagiIndexQuantization,
    CachingStrategy: CorticalAreaCachingStrategy,

// TODO neuron model as a typed enum for model with deeper fields
{
    pub metadata: GenomeObjectMetadata,
    pub model_data: (), // TODO Model Data Enum, Takes generic for async / sync for connectome data
    pub cache: CachingStrategy,


    _p: core::marker::PhantomData<FIQ>
}

//region Cache Data

/// What cortical level data is cached within the genome. Used for rapid lookups on genomes
/// that must be active
pub trait CorticalAreaCachingStrategy: Sized {
    fn build_cache(
        parent_region: BrainRegionID,
        incoming_mappings: HashSet<CorticalID>,
        outgoing_mappings: HashSet<CorticalID>
    ) -> Self;

    // TODO funcs for updating various properties that do nothign on no cache
}

/// A Caching stragedy of simply not caching. Useful for storage containers
pub struct CorticalAreaCachingDisabled;

/// Don't cache anything
impl CorticalAreaCachingStrategy for CorticalAreaCachingDisabled {
    fn build_cache(
        _parent_region: BrainRegionID,
        _incoming_mappings: HashSet<CorticalID>,
        _outgoing_mappings: HashSet<CorticalID>
    ) -> Self {
        CorticalAreaCachingDisabled
    }
}

/// Caches some data that is technically not needed but good for fast access.
pub struct CorticalAreaCachingEnabled {
    pub parent_region: BrainRegionID,
    pub incoming_mappings: HashSet<CorticalID>,
    pub outgoing_mappings: HashSet<CorticalID>,
}


impl CorticalAreaCachingStrategy for CorticalAreaCachingEnabled {
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



