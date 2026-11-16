use feagi_connectome::feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
use crate::basis::brain_region::BrainRegionID;
use crate::genome::genomes::brain_region::{BrainRegionGenomeContext, BrainRegionContainer, BrainRegionContainerRef, BrainRegionHierarchyContextRoot, BrainRegionSet, BrainRegionSetError};
use crate::genome::genomes::cortical_area::{CorticalAreaSet, GenomeCorticalAreaContext};

pub mod designer;
pub mod brain_region;
pub mod cortical_area;


/// Allows access to internal data states of a Genome Implementation
pub trait GenomePublicInterface<FIQ: FeagiIndexQuantization> {
    
    type BrainRegionSet: BrainRegionSet<BrainRegionContext=Self::BrainRegionContext>;
    type BrainRegionContext: BrainRegionGenomeContext;
    type CorticalAreaSet: CorticalAreaSet<FIQ>;
    
    fn get_brain_region_set(&self) -> &Self::BrainRegionSet;
    
    fn get_cortical_area_set(&self) -> &Self::CorticalAreaSet;
    
    //fn get_connectivity_rule_context(&self) -> Result<(), ()>;

    //fn get_cortical_mapping_set(&self) -> Result<(), ()>;

    //fn get_mapping_entry(&self) -> Result<(), ()>;

    //region default impls

    fn get_root_region_container(&self) -> &BrainRegionContainer<Self::BrainRegionContext, BrainRegionHierarchyContextRoot> {
        self.get_brain_region_set().get_root_brain_region_container()
    }

    fn get_brain_region_context(&self, region_id: &BrainRegionID) -> Result<BrainRegionContainerRef<Self::BrainRegionContext>, GenomeError> {
        let set = self.get_brain_region_set();
        set.get_brain_region_ref(region_id).ok_or_else(|| GenomeError::BrainRegion(BrainRegionSetError::BrainRegionIDNotFound))
    }

    //fn get_brain_region_ids_contained_by_region(&self) -> &HashSet<BrainRegionID>;

    //fn get_area_ids_contained_by_region(&self) -> &HashSet<CorticalIdentifier>;

    //fn get_cortical_area_context(&self, cortical_id: &CorticalIdentifier) -> Result<(), ()>;
    
    // TODO nested totals (cortical areas underneath brain region)?

    //endregion
}

/// Allows direct modification to genome structures that are accompanied by connectome level
/// changes. Since this often needs per implementation handling (including verification) this
/// must not be public. From outside the crate, these types of changes should only be done by
/// an external verifying system such as 'GenomeEditSequence'
pub(crate) trait GenomeInternalInterface<FIQ: FeagiIndexQuantization,>: GenomePublicInterface<FIQ> {
    
    // NOTE: In the case of designer, many of these changes should be made after any
    // applicable changes where made to the connectome with success first
    
    /// Gets brain regions in a mutable context. Cant be public since its possible to do things
    /// like drop cortical areas
    fn get_brain_region_contexts_mut(&mut self) -> &mut Self::BrainRegionSet;

    /// Gets cortical areas in a mutable context. Cant be public since its possible to do things
    /// like resize the area
    fn get_cortical_area_set_mut(&self) -> &mut Self::CorticalAreaSet;

    //fn get_connectivity_rule_contexts_mut(&mut self) -> ();

    // TODO mappings? should we take some sort of mappign summary?

    //region default impls


    fn apply_confirmed_changes(&mut self, )

    //endregion
}

#[derive(Debug, thiserror::Error)]
pub enum GenomeError {
    #[error(transparent)]
    BrainRegion(#[from] BrainRegionSetError),
}
