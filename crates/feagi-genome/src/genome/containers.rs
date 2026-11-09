use crate::basis::brain_region_identifiers::BrainRegionAncestryID;
use crate::genome::genome_object::{GenomeObjectMetadata, GenomeObjectPositioning};

pub trait BrainRegionContainer {

    fn is_root(&self) -> bool;

    fn get_metadata(&self) -> &GenomeObjectMetadata;

    fn get_metadata_mut(&mut self) -> &mut GenomeObjectMetadata;

    fn get_positioning(&self) -> &GenomeObjectPositioning;

    fn get_positioning_mut(&mut self) -> &mut GenomeObjectPositioning;

    fn get_ancestor_id(&self) -> &BrainRegionAncestryID;

    // No mut implementation of ancestor
}