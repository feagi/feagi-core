use crate::basis::brain_region_identifiers::{BrainRegionAncestryID, BrainRegionID};
use crate::genome::containers::BrainRegionContainer;
use crate::genome::genome_object::{GenomeObjectMetadata, GenomeObjectPositioning};

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
}