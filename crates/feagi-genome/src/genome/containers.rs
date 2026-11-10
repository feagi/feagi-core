use crate::basis::brain_region::{BrainRegionAncestryID, BrainRegionID};
use crate::basis::cortical_area::CorticalIdentifier;
use crate::genome::genome_object::GenomeObjectMetadata;
use std::collections::HashSet;

// Note: no positioning as the root region doesnt have it

pub trait BrainRegionContainer {
    fn is_root(&self) -> bool;

    fn get_metadata(&self) -> &GenomeObjectMetadata;

    fn get_ancestor_id(&self) -> &BrainRegionAncestryID;

    fn get_contained_regions(&self) -> &HashSet<BrainRegionID>;

    fn get_contained_cortical_areas(&self) -> &HashSet<CorticalIdentifier>;

    fn contains_brain_region(&self, child_id: &BrainRegionID) -> bool {
        self.get_contained_regions().contains(child_id)
    }
    
    fn contains_cortical_area(&self, cortical_area: &CorticalIdentifier) -> bool {
        self.get_contained_cortical_areas().contains(cortical_area)
    }
}

pub(crate) trait BrainRegionContainerMut: BrainRegionContainer  {

    fn get_metadata_mut(&mut self) -> &mut GenomeObjectMetadata;

    fn get_contained_regions_mut(&mut self) -> &mut HashSet<BrainRegionID>;

    fn get_contained_cortical_areas_mut(&mut self) -> &mut HashSet<CorticalIdentifier>;
}