use std::collections::HashSet;
use feagi_connectome::feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
use crate::basis::brain_region::BrainRegionID;
use crate::basis::cortical_area::cortical_identifiers::CorticalIdentifier;
use crate::genome::genomes::brain_region::{BrainRegionCachingContext, BrainRegionCachingContextCaching, BrainRegionCachingContextNone, BrainRegionContainer, BrainRegionContainerRef, BrainRegionHierarchyContextRoot, BrainRegionSet};

pub mod designer;
pub mod storage;
pub mod brain_region;

/// Allows reading of Genome implementation internal states in a straightforward manner. Reading is
/// safe so this can be publicly exposed
pub trait GenomeReadingInterface<FIQ: FeagiIndexQuantization, CC: CachingContext> {

    fn get_all_brain_regions(&self) -> &BrainRegionSet<CC::BrainRegion>; // root, hashmap of the rest // TODO a holder struct?

    //fn get_all_cortical_areas(&self) -> ();



    //fn get_connectivity_rule_context(&self) -> Result<(), ()>;

    //fn get_cortical_mapping_set(&self) -> Result<(), ()>;

    //fn get_mapping_entry(&self) -> Result<(), ()>;



    //region default impls

    fn get_root_region_container(&self) -> &BrainRegionContainer<CC::BrainRegion, BrainRegionHierarchyContextRoot> {
        self.get_all_brain_regions().get_root_brain_region_container()
    }

    fn get_brain_region_context(&self, region_id: &BrainRegionID) -> Result<BrainRegionContainerRef<CC::BrainRegion>, GenomeError> {
        let set = self.get_all_brain_regions();
        if let Some(region) = set.get_brain_region_ref(region_id) {
            Ok(region)
        } else {
            Err(GenomeError::InvalidRegionID)
        }
    }

    //fn get_brain_region_ids_contained_by_region(&self) -> &HashSet<BrainRegionID>;

    //fn get_area_ids_contained_by_region(&self) -> &HashSet<CorticalIdentifier>;

    //fn get_cortical_area_context(&self, cortical_id: &CorticalIdentifier) -> Result<(), ()>;





    // TODO nested totals (ortical areas underneath brain region)?

    //endregion
}

// TODO this seems useful for everything but designer? But also designer genome is not exposed publically soooo
pub trait GenomePublicModificationInterface<FIQ: FeagiIndexQuantization, CC: CachingContext>: GenomeInternalModificationInterface<FIQ, CC> {

    // TODO in the case of designer, these calls should also handle some sort of signaling. Maybe
    // a generic signaler should exist as a generic input to send updates to clients. This matters
    // only when clients support bv message!



}


/// Allows direct modification to genome structures that are accompanied by connectome level
/// changes. Since this often needs per implementation handling (including verification) this
/// must not be public. From outside the crate, these types of changes should only be done by
/// an external verifying system such as 'GenomeEditSequence'
pub(crate) trait GenomeInternalModificationInterface<FIQ: FeagiIndexQuantization, CC: CachingContext>: GenomeReadingInterface<FIQ, CC> {

    // NOTE: In the case of designer, many of these changes should be made after any
    // applicable changes where made to the connectome with success first

    // TODO in the case of designer, these calls should also handle some sort of signaling. Maybe
    // a generic signaler should exist as a generic input to send updates to clients. This matters
    // only when clients support bv message!

    /// Gets brain regions in a mutable context. Cant be public since its possible to do things
    /// like drop cortical areas
    fn get_brain_region_contexts_mut(&mut self) -> ();

    /// Gets cortical areas in a mutable context. Cant be public since its possible to do things
    /// like resize the area
    //fn get_cortical_area_contexts_mut(&mut self) -> ();

    //fn get_connectivity_rule_contexts_mut(&mut self) -> ();

    // TODO mappings? should we take some sort of mappign summary?

    //region default impls


    fn get_brain_region_context_mut(&mut self, region_id: &BrainRegionID) -> Result<(), ()>;

    //fn add_brain_region_context(&mut self) -> Result<(), ()>;


    // /// Gets the cortical area in a mutable context. Cant be public since its possible to do things
    // /// like resize the area
    //fn get_cortical_area_context_mut(&mut self, cortical_id: &CorticalIdentifier) -> Result<(), ()>;

    //fn add_cortical_area_context(&mut self) -> Result<(), ()>;


    //endregion
}


pub trait CachingContext {
    type BrainRegion: BrainRegionCachingContext;
}

pub struct CachingContextNone;

impl CachingContext for CachingContextNone {
    type BrainRegion =BrainRegionCachingContextNone;
}

pub struct CachingContextCaching;

impl CachingContext for CachingContextCaching {
    type BrainRegion = BrainRegionCachingContextCaching;
}


#[derive(Debug, thiserror::Error)]
pub enum GenomeError {
    #[error("Unable to find brain region of given ID!")]
    InvalidRegionID,
    #[error("Unable to the parent brain region of the given ID!")]
    InvalidParentRegionID,
    #[error("Given region ID already exists!")]
    DuplicateParentRegionID,
    #[error("This operation cannot be applied to the root region!")]
    InvalidRootIDOperation,
    #[error("This operation cannot be applied to a non-root region!")]
    InvalidInnerIDOperation,
}
