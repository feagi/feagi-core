use crate::identifiers::brain_regions::BrainRegionID;
use crate::identifiers::cortical_areas::CorticalID;
use std::collections::HashSet;

// TODO premade types 

pub struct CorticalArea<DataLookups, NeuronModel>
where
    DataLookups: CorticalAreaDataLookups,
// TODO neuron model as a typed enum for model with deeper fields
{
    pub name: String,
    pub description: String,
    pub position_3d: (),
    pub position_2d: (),

    pub cache_lookups: DataLookups,
    _t: core::marker::PhantomData<NeuronModel>
}

/// Describes how to get some data
pub trait CorticalAreaDataLookups {}

/// No additional data caching, useful for storage / export
pub struct CoritcalAreaStoredDataLookup;
impl CorticalAreaDataLookups for CoritcalAreaStoredDataLookup {}
impl CoritcalAreaStoredDataLookup {
    // TODO Sync Accessors
}

/// Caches some data that is technically not needed but good for fast access. Also has async accessors
/// for getting data from the NPU
pub struct CorticalAreaRuntimeCache {
    pub parent_region: BrainRegionID,
    pub incoming_mappings: HashSet<CorticalID>,
    pub outgoing_mappings: HashSet<CorticalID>,
}
impl CorticalAreaDataLookups for CorticalAreaRuntimeCache {}
impl CorticalAreaRuntimeCache {
    // TODO Async Accessors that use the NPU

    // TODO mapping lookups that use the runtime brain graph
}
