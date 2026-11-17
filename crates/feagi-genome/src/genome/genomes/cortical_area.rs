use std::collections::{HashMap, HashSet};
use feagi_connectome::connectome_basis::engine::BurstEngineIndex;
use feagi_connectome::connectome_basis::neuron_model::wrapped_indexing::CorticalAreaEngineIndex;
use feagi_connectome::connectome_basis::quantization_levels::FeagiIndexQuantization;
use feagi_connectome::connectome_basis::visualization::{CorticalAreaVisualizationContext, NeuronVisualizationVoxelByteIndex};
use crate::basis::brain_region::BrainRegionID;
use crate::basis::cortical_area::cortical_identifiers::CorticalIdentifier;
use crate::basis::cortical_area::{CorticalTypedGenerator, CorticalTypedStorage, CorticalTypedStorageOrGenerator};
use crate::genome::genome_object::{GenomeObjectMetadata, GenomeObjectPositioning};

pub trait CorticalAreaSet<FIQ: FeagiIndexQuantization> {
    type GenomeContext: GenomeCorticalAreaContext;
    type NeuronContext: StoredNeuronDataContext<FIQ>;
    
    fn new() -> Self;

    fn get_cortical_area(&self, cortical_id: &CorticalIdentifier)
        -> Option<&CorticalAreaContainer<FIQ, Self::GenomeContext, Self::NeuronContext>>;

    fn get_cortical_area_mut(&mut self, cortical_id: &CorticalIdentifier)
                         -> Option<&mut CorticalAreaContainer<FIQ, Self::GenomeContext, Self::NeuronContext>>;
}

pub trait GenomeCorticalAreaContext {}


//region Designer

pub struct CorticalAreaSetDesigner<FIQ: FeagiIndexQuantization> {
    areas: HashMap<CorticalIdentifier, CorticalAreaContainer<FIQ, GenomeCorticalAreaContextDesigner<FIQ>, CorticalTypedGenerator<FIQ> >>
}

impl<FIQ: FeagiIndexQuantization> CorticalAreaSet<FIQ> for CorticalAreaSetDesigner<FIQ> {
    type GenomeContext = GenomeCorticalAreaContextDesigner<FIQ>;
    type NeuronContext = CorticalTypedGenerator<FIQ>;

    fn new() -> Self {
        Self {
            areas: Default::default()
        }
    }

    fn get_cortical_area(&self, cortical_id: &CorticalIdentifier) -> Option<&CorticalAreaContainer<FIQ, Self::GenomeContext, Self::NeuronContext>> {
        self.areas.get(cortical_id)
    }

    fn get_cortical_area_mut(&mut self, cortical_id: &CorticalIdentifier) -> Option<&mut CorticalAreaContainer<FIQ, Self::GenomeContext, Self::NeuronContext>> {
        self.areas.get_mut(cortical_id)
    }
}


/// In the case of runtime, we will use memory to offer rapid lookups along with npu data caching
pub struct GenomeCorticalAreaContextDesigner<FIQ: FeagiIndexQuantization> {
    pub incoming_mappings: HashSet<CorticalIdentifier>,
    pub outgoing_mappings: HashSet<CorticalIdentifier>,
    /// What index is the cortical area saved under within the engine
    pub cortical_area_index: CorticalAreaEngineIndex<FIQ::CorticalAreaIndexCountQuant>,
    /// Where to start reading the information in the visualization feed
    pub visualization_array_read_start_index: CorticalAreaVisualizationContext<FIQ>,
    /// What engine is the cortical area under
    pub engine_index: BurstEngineIndex,
}

impl<FIQ: FeagiIndexQuantization> GenomeCorticalAreaContext for GenomeCorticalAreaContextDesigner<FIQ> {}

//endregion

//region Storage

/// In the case of dense storage, we dont want to store additional data
#[derive(Default, Clone)]
pub struct GenomeCorticalAreaContextStorage;

impl GenomeCorticalAreaContext for GenomeCorticalAreaContextStorage {}


//endregion

//region Contexts

/// If we only allow storage, Generator, or Either
pub trait StoredNeuronDataContext<FIQ: FeagiIndexQuantization> {}

impl<FIQ: FeagiIndexQuantization> StoredNeuronDataContext<FIQ> for CorticalTypedGenerator<FIQ> {}

impl<FIQ: FeagiIndexQuantization> StoredNeuronDataContext<FIQ> for CorticalTypedStorage<FIQ> {}

impl<FIQ: FeagiIndexQuantization> StoredNeuronDataContext<FIQ> for CorticalTypedStorageOrGenerator<FIQ> {}

//endregion


#[derive(Debug)]
pub struct CorticalAreaContainer<FIQ, CachingContext, NeuronDataContext>
where
    FIQ: FeagiIndexQuantization,
    CachingContext: GenomeCorticalAreaContext,
    NeuronDataContext: StoredNeuronDataContext<FIQ>,
{
    metadata: GenomeObjectMetadata,
    positioning: GenomeObjectPositioning,
    parent_id: BrainRegionID,
    cortical_data: NeuronDataContext,
    caching: CachingContext,
    _p: core::marker::PhantomData<FIQ>
}



