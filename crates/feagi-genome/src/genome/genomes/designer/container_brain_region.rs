use std::collections::HashSet;
use crate::basis::brain_region_identifiers::{BrainRegionAncestryID, BrainRegionID};
use crate::basis::cortical_area::CorticalIdentifier;
use crate::genome::containers::{BrainRegionContainer, BrainRegionContainerMut};
use crate::genome::genome_object::{GenomeObjectMetadata, GenomeObjectPositioning};
use crate::genome::genomes::GenomeError;

/// A borrowed brain region of the designer genome, either the root or an inner region
#[derive(Clone, Copy)]
pub enum BrainRegionContainerRuntimeEnumRef<'a> {
    Root(&'a BrainRegionContainerRootRuntime),
    Inner(&'a BrainRegionContainerInnerRuntime),
}

impl<'a> BrainRegionContainerRuntimeEnumRef<'a> {
    pub fn try_unwrap_as_root(&self) -> Result<&'a BrainRegionContainerRootRuntime, GenomeError> {
        match self {
            BrainRegionContainerRuntimeEnumRef::Root(container) => Ok(container),
            BrainRegionContainerRuntimeEnumRef::Inner(_) => Err(GenomeError::InvalidInnerIDOperation),
        }
    }

    pub fn try_unwrap_as_inner(&self) -> Result<&'a BrainRegionContainerInnerRuntime, GenomeError> {
        match self {
            BrainRegionContainerRuntimeEnumRef::Root(_) => Err(GenomeError::InvalidRootIDOperation),
            BrainRegionContainerRuntimeEnumRef::Inner(container) => Ok(container),
        }
    }

    /// The ancestry ID, borrowed for the full lifetime of the underlying region
    pub fn get_ancestor_id(&self) -> &'a BrainRegionAncestryID {
        match self {
            Self::Root(container) => container.get_ancestor_id(),
            Self::Inner(container) => container.get_ancestor_id(),
        }
    }
}

impl BrainRegionContainer for BrainRegionContainerRuntimeEnumRef<'_> {
    fn is_root(&self) -> bool {
        matches!(self, BrainRegionContainerRuntimeEnumRef::Root(_))
    }

    fn get_metadata(&self) -> &GenomeObjectMetadata {
        match self {
            BrainRegionContainerRuntimeEnumRef::Root(container) => container.get_metadata(),
            BrainRegionContainerRuntimeEnumRef::Inner(container) => container.get_metadata(),
        }
    }

    fn get_ancestor_id(&self) -> &BrainRegionAncestryID {
        match self {
            BrainRegionContainerRuntimeEnumRef::Root(container) => container.get_ancestor_id(),
            BrainRegionContainerRuntimeEnumRef::Inner(container) => container.get_ancestor_id(),
        }
    }

    fn get_contained_regions(&self) -> &HashSet<BrainRegionID> {
        match self {
            BrainRegionContainerRuntimeEnumRef::Root(container) => container.get_contained_regions(),
            BrainRegionContainerRuntimeEnumRef::Inner(container) => container.get_contained_regions(),
        }
    }

    fn get_contained_cortical_areas(&self) -> &HashSet<CorticalIdentifier> {
        match self {
            BrainRegionContainerRuntimeEnumRef::Root(container) => container.get_contained_cortical_areas(),
            BrainRegionContainerRuntimeEnumRef::Inner(container) => container.get_contained_cortical_areas(),
        }
    }
}

/// A mutably borrowed brain region of the designer genome, either the root or an inner region
pub enum BrainRegionContainerRuntimeEnumMut<'a> {
    Root(&'a mut BrainRegionContainerRootRuntime),
    Inner(&'a mut BrainRegionContainerInnerRuntime),
}

impl<'a> BrainRegionContainerRuntimeEnumMut<'a> {
    /// This region, borrowed immutably
    pub fn as_ref(&self) -> BrainRegionContainerRuntimeEnumRef<'_> {
        match self {
            Self::Root(container) => BrainRegionContainerRuntimeEnumRef::Root(container),
            Self::Inner(container) => BrainRegionContainerRuntimeEnumRef::Inner(container),
        }
    }

    pub fn try_unwrap_as_root_mut(&mut self) -> Result<&mut BrainRegionContainerRootRuntime, GenomeError> {
        match self {
            BrainRegionContainerRuntimeEnumMut::Root(container) => Ok(container),
            BrainRegionContainerRuntimeEnumMut::Inner(_) => Err(GenomeError::InvalidInnerIDOperation),
        }
    }

    pub fn try_unwrap_as_inner_mut(&mut self) -> Result<&mut BrainRegionContainerInnerRuntime, GenomeError> {
        match self {
            BrainRegionContainerRuntimeEnumMut::Root(_) => Err(GenomeError::InvalidRootIDOperation),
            BrainRegionContainerRuntimeEnumMut::Inner(container) => Ok(container),
        }
    }

    pub(crate) fn insert_contained_brain_region_id(&mut self, contained_id: BrainRegionID) { // TODO error for collision
        self.get_contained_regions_mut().insert(contained_id);
    }
}

impl BrainRegionContainer for BrainRegionContainerRuntimeEnumMut<'_> {
    fn is_root(&self) -> bool {
        matches!(self, BrainRegionContainerRuntimeEnumMut::Root(_))
    }

    fn get_metadata(&self) -> &GenomeObjectMetadata {
        match self {
            BrainRegionContainerRuntimeEnumMut::Root(container) => container.get_metadata(),
            BrainRegionContainerRuntimeEnumMut::Inner(container) => container.get_metadata(),
        }
    }

    fn get_ancestor_id(&self) -> &BrainRegionAncestryID {
        match self {
            BrainRegionContainerRuntimeEnumMut::Root(container) => container.get_ancestor_id(),
            BrainRegionContainerRuntimeEnumMut::Inner(container) => container.get_ancestor_id(),
        }
    }

    fn get_contained_regions(&self) -> &HashSet<BrainRegionID> {
        match self {
            BrainRegionContainerRuntimeEnumMut::Root(container) => container.get_contained_regions(),
            BrainRegionContainerRuntimeEnumMut::Inner(container) => container.get_contained_regions(),
        }
    }

    fn get_contained_cortical_areas(&self) -> &HashSet<CorticalIdentifier> {
        match self {
            BrainRegionContainerRuntimeEnumMut::Root(container) => container.get_contained_cortical_areas(),
            BrainRegionContainerRuntimeEnumMut::Inner(container) => container.get_contained_cortical_areas(),
        }
    }
}

impl BrainRegionContainerMut for BrainRegionContainerRuntimeEnumMut<'_> {
    fn get_metadata_mut(&mut self) -> &mut GenomeObjectMetadata {
        match self {
            BrainRegionContainerRuntimeEnumMut::Root(container) => container.get_metadata_mut(),
            BrainRegionContainerRuntimeEnumMut::Inner(container) => container.get_metadata_mut(),
        }
    }

    fn get_contained_regions_mut(&mut self) -> &mut HashSet<BrainRegionID> {
        match self {
            BrainRegionContainerRuntimeEnumMut::Root(container) => container.get_contained_regions_mut(),
            BrainRegionContainerRuntimeEnumMut::Inner(container) => container.get_contained_regions_mut(),
        }
    }

    fn get_contained_cortical_areas_mut(&mut self) -> &mut HashSet<CorticalIdentifier> {
        match self {
            BrainRegionContainerRuntimeEnumMut::Root(container) => container.get_contained_cortical_areas_mut(),
            BrainRegionContainerRuntimeEnumMut::Inner(container) => container.get_contained_cortical_areas_mut(),
        }
    }
}

pub(crate) struct BrainRegionContainerRootRuntime {
    pub(crate) root_id: BrainRegionID,
    pub(crate) metadata: GenomeObjectMetadata,
    pub(crate) ancestor_id: BrainRegionAncestryID,
    pub(crate) contained_areas: HashSet<CorticalIdentifier>,
    pub(crate) contained_regions: HashSet<BrainRegionID>
}

impl BrainRegionContainerRootRuntime {
    pub(crate) fn new_now(metadata: GenomeObjectMetadata) -> (Self, BrainRegionID) {
        let this_id = BrainRegionID::new_now();
        (Self {
            root_id: this_id.clone(),
            metadata,
            ancestor_id: this_id.into(),
            contained_areas: Default::default(),
            contained_regions: Default::default(),
        },
            this_id
        )
    }
}

impl BrainRegionContainer for BrainRegionContainerRootRuntime {
    fn is_root(&self) -> bool {
        true
    }

    fn get_metadata(&self) -> &GenomeObjectMetadata {
        &self.metadata
    }

    fn get_ancestor_id(&self) -> &BrainRegionAncestryID {
        &self.ancestor_id
    }

    fn get_contained_regions(&self) -> &HashSet<BrainRegionID> {
        &self.contained_regions
    }

    fn get_contained_cortical_areas(&self) -> &HashSet<CorticalIdentifier> {
        &self.contained_areas
    }
}

impl BrainRegionContainerMut for BrainRegionContainerRootRuntime {
    fn get_metadata_mut(&mut self) -> &mut GenomeObjectMetadata {
        &mut self.metadata
    }

    fn get_contained_regions_mut(&mut self) -> &mut HashSet<BrainRegionID> {
        &mut self.contained_regions
    }

    fn get_contained_cortical_areas_mut(&mut self) -> &mut HashSet<CorticalIdentifier> {
        &mut self.contained_areas
    }
}


pub(crate) struct BrainRegionContainerInnerRuntime {
    pub(crate) metadata: GenomeObjectMetadata,
    pub(crate) positioning: GenomeObjectPositioning,
    pub(crate) parent_id: BrainRegionID,
    pub(crate) ancestor_id: BrainRegionAncestryID,
    pub(crate) contained_areas: HashSet<CorticalIdentifier>,
    pub(crate) contained_regions: HashSet<BrainRegionID>
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

    pub fn get_positioning(&self) -> &GenomeObjectPositioning {
        &self.positioning
    }

    pub fn get_positioning_mut(&mut self) -> &mut GenomeObjectPositioning {
        &mut self.positioning
    }
}

impl BrainRegionContainer for BrainRegionContainerInnerRuntime {
    fn is_root(&self) -> bool {
        false
    }

    fn get_metadata(&self) -> &GenomeObjectMetadata {
        &self.metadata
    }

    fn get_ancestor_id(&self) -> &BrainRegionAncestryID {
        &self.ancestor_id
    }

    fn get_contained_regions(&self) -> &HashSet<BrainRegionID> {
        &self.contained_regions
    }

    fn get_contained_cortical_areas(&self) -> &HashSet<CorticalIdentifier> {
        &self.contained_areas
    }
}

impl BrainRegionContainerMut for BrainRegionContainerInnerRuntime {
    fn get_metadata_mut(&mut self) -> &mut GenomeObjectMetadata {
        &mut self.metadata
    }

    fn get_contained_regions_mut(&mut self) -> &mut HashSet<BrainRegionID> {
        &mut self.contained_regions
    }

    fn get_contained_cortical_areas_mut(&mut self) -> &mut HashSet<CorticalIdentifier> {
        &mut self.contained_areas
    }
}
