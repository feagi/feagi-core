use std::collections::HashSet;
use crate::basis::brain_region_identifiers::{BrainRegionAncestryID, BrainRegionID};
use crate::basis::cortical_area::CorticalIdentifier;
use crate::genome::containers::BrainRegionContainer;
use crate::genome::genome_object::{GenomeObjectMetadata, GenomeObjectPositioning};

//region Runtime
pub enum BrainRegionContainerRuntime {
    Root(BrainRegionContainerRootRuntime),
    Inner(BrainRegionContainerInnerRuntime),
}

impl BrainRegionContainerRuntime {
    /// The cortical areas directly inside this region
    pub fn get_contained_areas(&self) -> &HashSet<CorticalIdentifier> {
        match self {
            Self::Root(container) => container.get_contained_areas(),
            Self::Inner(container) => container.get_contained_areas(),
        }
    }

    /// The cortical areas directly inside this region
    pub fn get_contained_areas_mut(&mut self) -> &mut HashSet<CorticalIdentifier> {
        match self {
            Self::Root(container) => container.get_contained_areas_mut(),
            Self::Inner(container) => container.get_contained_areas_mut(),
        }
    }

    /// The brain regions directly inside this region
    pub fn get_contained_regions(&self) -> &HashSet<BrainRegionID> {
        match self {
            Self::Root(container) => container.get_contained_regions(),
            Self::Inner(container) => container.get_contained_regions(),
        }
    }

    /// The brain regions directly inside this region
    pub fn get_contained_regions_mut(&mut self) -> &mut HashSet<BrainRegionID> {
        match self {
            Self::Root(container) => container.get_contained_regions_mut(),
            Self::Inner(container) => container.get_contained_regions_mut(),
        }
    }
}

impl BrainRegionContainer for BrainRegionContainerRuntime {
    fn is_root(&self) -> bool {
        match self {
            Self::Root(container) => container.is_root(),
            Self::Inner(container) => container.is_root(),
        }
    }

    fn get_metadata(&self) -> &GenomeObjectMetadata {
        match self {
            Self::Root(container) => container.get_metadata(),
            Self::Inner(container) => container.get_metadata(),
        }
    }

    fn get_metadata_mut(&mut self) -> &mut GenomeObjectMetadata {
        match self {
            Self::Root(container) => container.get_metadata_mut(),
            Self::Inner(container) => container.get_metadata_mut(),
        }
    }

    fn get_positioning(&self) -> &GenomeObjectPositioning {
        match self {
            Self::Root(container) => container.get_positioning(),
            Self::Inner(container) => container.get_positioning(),
        }
    }

    fn get_positioning_mut(&mut self) -> &mut GenomeObjectPositioning {
        match self {
            Self::Root(container) => container.get_positioning_mut(),
            Self::Inner(container) => container.get_positioning_mut(),
        }
    }

    fn get_ancestor_id(&self) -> &BrainRegionAncestryID {
        match self {
            Self::Root(container) => container.get_ancestor_id(),
            Self::Inner(container) => container.get_ancestor_id(),
        }
    }
}

pub struct BrainRegionContainerRootRuntime {
    pub metadata: GenomeObjectMetadata,
    pub positioning: GenomeObjectPositioning,
    pub ancestor_id: BrainRegionAncestryID,
    pub contained_areas: HashSet<CorticalIdentifier>,
    pub contained_regions: HashSet<BrainRegionID>
}

impl BrainRegionContainerRootRuntime {
    /// The cortical areas directly inside this region
    pub fn get_contained_areas(&self) -> &HashSet<CorticalIdentifier> {
        &self.contained_areas
    }

    /// The cortical areas directly inside this region
    pub fn get_contained_areas_mut(&mut self) -> &mut HashSet<CorticalIdentifier> {
        &mut self.contained_areas
    }

    /// The brain regions directly inside this region
    pub fn get_contained_regions(&self) -> &HashSet<BrainRegionID> {
        &self.contained_regions
    }

    /// The brain regions directly inside this region
    pub fn get_contained_regions_mut(&mut self) -> &mut HashSet<BrainRegionID> {
        &mut self.contained_regions
    }
}

impl BrainRegionContainer for BrainRegionContainerRootRuntime {
    fn is_root(&self) -> bool {
        true
    }

    fn get_metadata(&self) -> &GenomeObjectMetadata {
        &self.metadata
    }

    fn get_metadata_mut(&mut self) -> &mut GenomeObjectMetadata {
        &mut self.metadata
    }

    fn get_positioning(&self) -> &GenomeObjectPositioning {
        &self.positioning
    }

    fn get_positioning_mut(&mut self) -> &mut GenomeObjectPositioning {
        &mut self.positioning
    }

    fn get_ancestor_id(&self) -> &BrainRegionAncestryID {
        &self.ancestor_id
    }
}

pub struct BrainRegionContainerInnerRuntime {
    pub metadata: GenomeObjectMetadata,
    pub positioning: GenomeObjectPositioning,
    pub parent_id: BrainRegionID,
    pub ancestor_id: BrainRegionAncestryID,
    pub contained_areas: HashSet<CorticalIdentifier>,
    pub contained_regions: HashSet<BrainRegionID>
}

impl BrainRegionContainerInnerRuntime {
    /// The cortical areas directly inside this region
    pub fn get_contained_areas(&self) -> &HashSet<CorticalIdentifier> {
        &self.contained_areas
    }

    /// The cortical areas directly inside this region
    pub fn get_contained_areas_mut(&mut self) -> &mut HashSet<CorticalIdentifier> {
        &mut self.contained_areas
    }

    /// The brain regions directly inside this region
    pub fn get_contained_regions(&self) -> &HashSet<BrainRegionID> {
        &self.contained_regions
    }

    /// The brain regions directly inside this region
    pub fn get_contained_regions_mut(&mut self) -> &mut HashSet<BrainRegionID> {
        &mut self.contained_regions
    }
}

impl BrainRegionContainer for BrainRegionContainerInnerRuntime {
    fn is_root(&self) -> bool {
        false
    }

    fn get_metadata(&self) -> &GenomeObjectMetadata {
        &self.metadata
    }

    fn get_metadata_mut(&mut self) -> &mut GenomeObjectMetadata {
        &mut self.metadata
    }

    fn get_positioning(&self) -> &GenomeObjectPositioning {
        &self.positioning
    }

    fn get_positioning_mut(&mut self) -> &mut GenomeObjectPositioning {
        &mut self.positioning
    }

    fn get_ancestor_id(&self) -> &BrainRegionAncestryID {
        &self.ancestor_id
    }
}