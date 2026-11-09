use std::collections::HashSet;
use crate::basis::brain_region_identifiers::{BrainRegionAncestryID, BrainRegionID};
use crate::basis::cortical_area::CorticalIdentifier;
use crate::genome::containers::BrainRegionContainer;
use crate::genome::genome_object::{GenomeObjectMetadata, GenomeObjectPositioning};
use crate::genome::genomes::GenomeError;

/// A borrowed brain region of the designer genome, either the root or an inner region
#[derive(Clone, Copy)]
pub enum BrainRegionContainerRuntimeRef<'a> {
    Root(&'a BrainRegionContainerRootRuntime),
    Inner(&'a BrainRegionContainerInnerRuntime),
}

impl<'a> BrainRegionContainerRuntimeRef<'a> {
    /// If this is the root region
    pub fn is_root(&self) -> bool {
        match self {
            Self::Root(container) => true,
            Self::Inner(container) => false,
        }
    }
    
    pub fn unwrap_as_inner(&self) -> Result<&BrainRegionContainerInnerRuntime, GenomeError> {
        match self {
            BrainRegionContainerRuntimeRef::Root(r) => {
                Err(GenomeError::InvalidRootIDOperation)
            }
            BrainRegionContainerRuntimeRef::Inner(i) => {
                Ok(i)
            }
        }
    }

    pub fn get_metadata(&self) -> &'a GenomeObjectMetadata {
        match self {
            Self::Root(container) => container.get_metadata(),
            Self::Inner(container) => container.get_metadata(),
        }
    }

    pub fn get_positioning(&self) -> &'a GenomeObjectPositioning {
        match self {
            Self::Root(container) => container.get_positioning(),
            Self::Inner(container) => container.get_positioning(),
        }
    }

    pub fn get_ancestor_id(&self) -> &'a BrainRegionAncestryID {
        match self {
            Self::Root(container) => container.get_ancestor_id(),
            Self::Inner(container) => container.get_ancestor_id(),
        }
    }

    /// The cortical areas directly inside this region
    pub fn get_contained_areas(&self) -> &'a HashSet<CorticalIdentifier> {
        match self {
            Self::Root(container) => container.get_contained_areas(),
            Self::Inner(container) => container.get_contained_areas(),
        }
    }

    /// The brain regions directly inside this region
    pub fn get_contained_regions(&self) -> &'a HashSet<BrainRegionID> {
        match self {
            Self::Root(container) => container.get_contained_regions(),
            Self::Inner(container) => container.get_contained_regions(),
        }
    }
}

/// A mutably borrowed brain region of the designer genome, either the root or an inner region
pub enum BrainRegionContainerRuntimeMut<'a> {
    Root(&'a mut BrainRegionContainerRootRuntime),
    Inner(&'a mut BrainRegionContainerInnerRuntime),
}

impl<'a> BrainRegionContainerRuntimeMut<'a> {
    /// This region, borrowed immutably
    pub fn as_ref(&self) -> BrainRegionContainerRuntimeRef<'_> {
        match self {
            Self::Root(container) => BrainRegionContainerRuntimeRef::Root(container),
            Self::Inner(container) => BrainRegionContainerRuntimeRef::Inner(container),
        }
    }

    /// If this is the root region
    pub fn is_root(&self) -> bool {
        match self {
            Self::Root(container) => true,
            Self::Inner(container) => false,
        }
    }

    pub fn unwrap_as_inner(&self) -> Result<&mut BrainRegionContainerInnerRuntime, GenomeError> {
        match self {
            BrainRegionContainerRuntimeMut::Root(r) => {
                Err(GenomeError::InvalidRootIDOperation)
            }
            BrainRegionContainerRuntimeMut::Inner(i) => {
                Ok(*i)
            }
        }
    }
    
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

    pub(crate) fn insert_contained_brain_region_id(&mut self, contained_id: BrainRegionID) { // TODO error for collision
        self.get_contained_regions_mut().insert(contained_id);
    }
}

impl BrainRegionContainer for BrainRegionContainerRuntimeMut<'_> {
    fn is_root(&self) -> bool {
        self.as_ref().is_root()
    }

    fn get_metadata(&self) -> &GenomeObjectMetadata {
        self.as_ref().get_metadata()
    }

    fn get_metadata_mut(&mut self) -> &mut GenomeObjectMetadata {
        match self {
            Self::Root(container) => container.get_metadata_mut(),
            Self::Inner(container) => container.get_metadata_mut(),
        }
    }

    fn get_positioning(&self) -> &GenomeObjectPositioning {
        self.as_ref().get_positioning()
    }

    fn get_positioning_mut(&mut self) -> &mut GenomeObjectPositioning {
        match self {
            Self::Root(container) => container.get_positioning_mut(),
            Self::Inner(container) => container.get_positioning_mut(),
        }
    }

    fn get_ancestor_id(&self) -> &BrainRegionAncestryID {
        self.as_ref().get_ancestor_id()
    }
}

pub struct BrainRegionContainerRootRuntime {
    pub root_id: BrainRegionID,
    pub metadata: GenomeObjectMetadata,
    pub positioning: GenomeObjectPositioning,
    pub ancestor_id: BrainRegionAncestryID,
    pub contained_areas: HashSet<CorticalIdentifier>,
    pub contained_regions: HashSet<BrainRegionID>
}

impl BrainRegionContainerRootRuntime {
    
    pub(crate) fn new_now(metadata: GenomeObjectMetadata) -> (Self, BrainRegionID) {
        let this_id = BrainRegionID::new_now();
        (Self {
            root_id: this_id.clone(),
            metadata,
            positioning: GenomeObjectPositioning::default(),
            ancestor_id: this_id.into(),
            contained_areas: Default::default(),
            contained_regions: Default::default(),
        },
            this_id
        )
    }
    
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
    pub fn new_now(parent_id: BrainRegionID, metadata: GenomeObjectMetadata, positioning: GenomeObjectPositioning) -> (Self, BrainRegionID) {
        let this_id = BrainRegionID::new_now();

        (Self {
            metadata,
            positioning,
            parent_id,
            ancestor_id: this_id.clone().into(),
            contained_areas: Default::default(),
            contained_regions: Default::default(),
        },
         this_id
        )
    }
    
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