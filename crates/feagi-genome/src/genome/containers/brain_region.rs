use std::collections::HashSet;
use crate::basis::cortical_area::cortical_identifier::CorticalIdentifier;
use crate::basis::specifications::brain_regions::{BrainRegionAncestryID, BrainRegionID};
use crate::genome::genome_object_metadata::GenomeObjectMetadata;
use crate::genome::genome_object_positioning::GenomeObjectPositioning;


pub trait BrainRegionContainer {

    fn is_root(&self) -> bool;

    fn get_metadata(&self) -> &GenomeObjectMetadata;

    fn get_metadata_mut(&mut self) -> &mut GenomeObjectMetadata;

    fn get_positioning(&self) -> &GenomeObjectPositioning;

    fn get_positioning_mut(&mut self) -> &mut GenomeObjectPositioning;

    fn get_ancestor_id(&self) -> &BrainRegionAncestryID;

    fn get_ancestor_id_mut(&mut self) -> &mut BrainRegionAncestryID;
}

//region Runtime
pub enum BrainRegionContainerRuntime {
    Root(BrainRegionContainerRootRuntime),
    Inner(BrainRegionContainerInnerRuntime),
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

    fn get_ancestor_id_mut(&mut self) -> &mut BrainRegionAncestryID {
        match self {
            Self::Root(container) => container.get_ancestor_id_mut(),
            Self::Inner(container) => container.get_ancestor_id_mut(),
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

    fn get_ancestor_id_mut(&mut self) -> &mut BrainRegionAncestryID {
        &mut self.ancestor_id
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

    fn get_ancestor_id_mut(&mut self) -> &mut BrainRegionAncestryID {
        &mut self.ancestor_id
    }
}

//endregion

//region Storage
pub enum BrainRegionContainerStorage {
    Root(BrainRegionContainerRootStorage),
    Inner(BrainRegionContainerInnerStorage),
}

impl BrainRegionContainer for BrainRegionContainerStorage {
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

    fn get_ancestor_id_mut(&mut self) -> &mut BrainRegionAncestryID {
        match self {
            Self::Root(container) => container.get_ancestor_id_mut(),
            Self::Inner(container) => container.get_ancestor_id_mut(),
        }
    }
}

pub struct BrainRegionContainerRootStorage {
    pub metadata: GenomeObjectMetadata,
    pub positioning: GenomeObjectPositioning,
    pub ancestor_id: BrainRegionAncestryID,
}

impl BrainRegionContainer for BrainRegionContainerRootStorage {
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

    fn get_ancestor_id_mut(&mut self) -> &mut BrainRegionAncestryID {
        &mut self.ancestor_id
    }
}

pub struct BrainRegionContainerInnerStorage {
    pub metadata: GenomeObjectMetadata,
    pub positioning: GenomeObjectPositioning,
    pub parent_id: BrainRegionID,
    pub ancestor_id: BrainRegionAncestryID,
}

impl BrainRegionContainer for BrainRegionContainerInnerStorage {
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

    fn get_ancestor_id_mut(&mut self) -> &mut BrainRegionAncestryID {
        &mut self.ancestor_id
    }
}

//endregion


