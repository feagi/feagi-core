use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use feagi_connectome::feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
use crate::basis::brain_region::BrainRegionID;
use crate::basis::cortical_area::cortical_identifiers::CorticalIdentifier;
use crate::basis::cortical_area::{CorticalTypedGeneratorPrecursor, CorticalTypedStorage, CorticalTypedStorageOrGeneratorPrecursor};
use crate::genome::genome_object::{GenomeObjectMetadata, GenomeObjectPositioning};

pub struct CorticalAreaSet<FIQ, CachingContext, NeuronDataContext>
where
    FIQ: FeagiIndexQuantization,
    CachingContext: CorticalAreaCachingContext,
    NeuronDataContext: StoredNeuronDataContext <FIQ>
{
    areas: HashMap<CorticalIdentifier, CorticalAreaContainer<FIQ, CachingContext, NeuronDataContext>>
}

impl<FIQ, CachingContext, NeuronDataContext> CorticalAreaSet<FIQ, CachingContext, NeuronDataContext>
where
    FIQ: FeagiIndexQuantization,
    CachingContext: CorticalAreaCachingContext,
    NeuronDataContext: StoredNeuronDataContext <FIQ>
{
    
    pub fn get_cortical_area(&self, cortical_id: &CorticalIdentifier)
        -> Option<&CorticalAreaContainer<FIQ, CachingContext, NeuronDataContext>> {
        self.areas.get(cortical_id)
    }

    pub fn get_cortical_area_mut(&mut self, cortical_id: &CorticalIdentifier)
                             -> Option<&mut CorticalAreaContainer<FIQ, CachingContext, NeuronDataContext>> {
        self.areas.get_mut(cortical_id)
    }


}


impl<FIQ, CachingContext, NeuronDataContext> Default
for CorticalAreaSet<FIQ, CachingContext, NeuronDataContext>
where
    FIQ: FeagiIndexQuantization,
    CachingContext: CorticalAreaCachingContext,
    NeuronDataContext: StoredNeuronDataContext <FIQ>
{
    fn default() -> Self {
        Self {
            areas: Default::default()
        }
    }
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


/// If we only allow storage, Generator, or Either
pub trait StoredNeuronDataContext<FIQ: FeagiIndexQuantization> {

}

impl<FIQ: FeagiIndexQuantization> StoredNeuronDataContext<FIQ> for CorticalTypedGeneratorPrecursor<FIQ> {}

impl<FIQ: FeagiIndexQuantization> StoredNeuronDataContext<FIQ> for CorticalTypedStorage<FIQ> {}

impl<FIQ: FeagiIndexQuantization> StoredNeuronDataContext<FIQ> for CorticalTypedStorageOrGeneratorPrecursor<FIQ> {}

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
    cortical_data: NeuronDataContext,
    caching: CachingContext,
    _p: core::marker::PhantomData<FIQ>
}



