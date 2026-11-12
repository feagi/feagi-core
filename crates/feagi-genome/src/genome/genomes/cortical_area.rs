use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use feagi_connectome::feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
use crate::basis::brain_region::BrainRegionID;
use crate::basis::cortical_area::cortical_identifiers::CorticalIdentifier;
use crate::basis::cortical_area::CTGeneratorTyped;
use crate::genome::genome_object::{GenomeObjectMetadata, GenomeObjectPositioning};

pub struct CorticalAreaSet<FIQ: FeagiIndexQuantization, CachingContext: CorticalAreaCachingContext> {
    areas: HashMap<CorticalIdentifier, CorticalAreaContainer<FIQ, CachingContext>>
}



//region Contexts

pub trait CorticalAreaCachingContext: Default {}

/// In the case of dense storage, we dont want to store additional data
#[derive(Default, Clone)]
pub struct CorticalAreaCachingContextNone;

impl CorticalAreaCachingContext for CorticalAreaCachingContextNone {}

/// In the case of runtime, we will use memory to offer rapid lookups
#[derive(Default)]
pub struct CorticalAreaCachingContextCaching {
    pub incoming_mappings: HashSet<CorticalIdentifier>,
    pub outgoing_mappings: HashSet<CorticalIdentifier>,
}

impl CorticalAreaCachingContext for CorticalAreaCachingContextCaching {}



pub trait StoredNeuronDataContext<FIQ: FeagiIndexQuantization>: Default {
    /// If we only allow storage, Generator, or Either
    type NeurogenesisType<FIQ>;
}


//endregion





#[derive(Debug)]
pub struct CorticalAreaContainer<FIQ, CachingContext, NeuronDataContext>
where
    FIQ: FeagiIndexQuantization,
    CachingContext: CorticalAreaCachingContext,
    NeuronDataContext: StoredNeuronDataContext<FIQ>,
{
    metadata: GenomeObjectMetadata,
    positioning: GenomeObjectPositioning,
    parent_id: BrainRegionID,
    genome_generator: CTGeneratorTyped<FIQ>, // As long as its not a storage type, this needs to be small
    caching: CachingContext,
    _p: core::marker::PhantomData<NeuronDataContext>
}



