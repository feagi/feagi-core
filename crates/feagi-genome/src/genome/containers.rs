use std::collections::HashSet;
use crate::basis::brain_region_identifiers::{BrainRegionAncestryID, BrainRegionID};
use crate::basis::cortical_area::CorticalIdentifier;
use crate::genome::genome_object::{GenomeObjectMetadata, GenomeObjectPositioning};
use crate::genome::genomes::designer::container_brain_region::BrainRegionContainerInnerRuntime;
use crate::genome::genomes::GenomeError;

pub trait BrainRegionContainer {

    fn is_root(&self) -> bool;

    fn get_metadata(&self) -> &GenomeObjectMetadata;
    
    fn get_positioning(&self) -> &GenomeObjectPositioning;

    fn get_ancestor_id(&self) -> &BrainRegionAncestryID;
    
    fn contains_brain_region(&self, child_id: &BrainRegionID) -> bool;
    
    fn contains_cortical_area(&self, cortical_area: &CorticalIdentifier) -> bool;
}

