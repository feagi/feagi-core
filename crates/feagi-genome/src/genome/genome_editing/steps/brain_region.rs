use serde::{Deserialize, Serialize};
use crate::basis::brain_region::{BrainRegionAncestryID, BrainRegionID};
use crate::genome::genome_object::{GenomeObjectMetadata, GenomeObjectPositioning};




/// The enum edit step for brain regions for the load genome sequence type. Can only add brain regions
pub enum GenomeEditStepBrainRegionAddingOnlyEnum {
    /// Adds a brain region with the given details. if optional_parent_region is None, the region is
    /// added to the root brain region
    AddRegion {
        metadata: GenomeObjectMetadata,
        positioning: GenomeObjectPositioning,
        this_brain_region_id: BrainRegionID,
        this_brain_region_ancestry: BrainRegionAncestryID,
        optional_parent_region: Option<BrainRegionID>,
    }
}




/// The enum edit step for brain regions for editing brain regions in any way
pub enum GenomeEditStepBrainRegionEditingEnum {
    /// Adds a brain region with the given details. if optional_parent_region is None, the region is
    /// added to the root brain region
    AddRegion {
        metadata: GenomeObjectMetadata,
        positioning: GenomeObjectPositioning,
        this_brain_region_id: BrainRegionID,
        this_brain_region_ancestry: BrainRegionAncestryID,
        optional_parent_region: Option<BrainRegionID>,
    },
    /// Deletes a given inner region. Anything inside gets moved up first
    DeleteRegion {
        deleting: BrainRegionID
    }
    // TODO others
}

//endregion


//endregion



#[derive(Debug, thiserror::Error, Clone, Copy, Serialize, Deserialize)]
pub enum GenomeEditStepBrainRegionError {
    #[error("A Brain region with the given ID is already being added!")]
    BrainRegionWithIDAlreadyBeingAdded,
    #[error("A Brain region with the given ancestryID is already being added!")]
    BrainRegionWithAncestryIDAlreadyBeingAdded,
    // TODO fill with errors
    #[error("TODO")]
    Unknown,
}