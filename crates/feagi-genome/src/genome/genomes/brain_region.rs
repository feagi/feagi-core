use crate::basis::brain_region::{BrainRegionAncestryID, BrainRegionID, BrainRegionIdAndAncestryConstructor};
use crate::basis::cortical_area::cortical_identifiers::CorticalIdentifier;
use crate::genome::genome_object::{GenomeObjectMetadata, GenomeObjectPositioning};
use std::collections::{HashMap, HashSet};

/// Defines holding a brain region set of data
pub trait BrainRegionSet: Sized {
    type BrainRegionContext: BrainRegionGenomeContext;

    fn new_brain_region_set(metadata: GenomeObjectMetadata, ids: BrainRegionIdAndAncestryConstructor) -> Self;

    fn contains_ancestry_id(&self, id: &BrainRegionAncestryID) -> bool;

    fn get_root_brain_region_container(&self) -> &BrainRegionContainer<Self::BrainRegionContext, BrainRegionHierarchyContextRoot>;

    fn get_root_brain_region_container_mut(&mut self) -> &mut BrainRegionContainer<Self::BrainRegionContext, BrainRegionHierarchyContextRoot>;

    fn get_inner_brain_region_container(
        &self,
        brain_region_id: &BrainRegionID,
    ) -> Option<&BrainRegionContainer<Self::BrainRegionContext, BrainRegionHierarchyContextInner>>;

    fn get_inner_brain_region_container_mut(
        &mut self,
        brain_region_id: &BrainRegionID,
    ) -> Option<&mut BrainRegionContainer<Self::BrainRegionContext, BrainRegionHierarchyContextInner>>;

    fn get_brain_region_ref(&self, brain_region_id: &BrainRegionID) -> Option<BrainRegionContainerRef<Self::BrainRegionContext>>;

    fn get_brain_region_ref_mut(&mut self, brain_region_id: &BrainRegionID) -> Option<BrainRegionContainerRefMut<Self::BrainRegionContext>>;

    fn add_new_inner_brain_region(
        &mut self,
        ids: BrainRegionIdAndAncestryConstructor,
        parent_id: &BrainRegionID,
        metadata: GenomeObjectMetadata,
        positioning: GenomeObjectPositioning,
    ) -> Result<(), BrainRegionSetError>;

    fn remove_inner_brain_region(&mut self, container_id: &BrainRegionID) -> Result<(), BrainRegionSetError>;

    fn move_inner_brain_region(
        &mut self,
        moving: Self::BrainRegionContext,
        from: &BrainRegionID,
        to: &BrainRegionID,
    ) -> Result<(), BrainRegionSetError>;

    fn add_cortical_area(&mut self, cortical_id: CorticalIdentifier, parent_id: &BrainRegionID) -> Result<(), BrainRegionSetError>;

    fn move_cortical_area(&mut self, moving: Self::BrainRegionContext, from: &BrainRegionID, to: &BrainRegionID) -> Result<(), BrainRegionSetError>;
}

pub struct BrainRegionSetDesigner {
    root_brain_region: BrainRegionContainer<BrainRegionContextDesigner, BrainRegionHierarchyContextRoot>,
    inner_brain_regions: HashMap<BrainRegionID, BrainRegionContainer<BrainRegionContextDesigner, BrainRegionHierarchyContextInner>>,
    contained_ancestry_ids: HashSet<BrainRegionAncestryID>,
}

impl BrainRegionSet for BrainRegionSetDesigner {
    type BrainRegionContext = BrainRegionContextDesigner;

    fn new_brain_region_set(metadata: GenomeObjectMetadata, ids: BrainRegionIdAndAncestryConstructor) -> Self {
        let (root_id, root_ancestry) = ids.into_pair();
        let root_brain_region = BrainRegionContainer::new_root(root_ancestry, metadata, root_id);
        Self {
            root_brain_region,
            inner_brain_regions: Default::default(),
            contained_ancestry_ids: Default::default(),
        }
    }

    fn contains_ancestry_id(&self, id: &BrainRegionAncestryID) -> bool {
        self.contained_ancestry_ids.contains(id)
    }

    fn get_root_brain_region_container(&self) -> &BrainRegionContainer<Self::BrainRegionContext, BrainRegionHierarchyContextRoot> {
        &self.root_brain_region
    }

    fn get_root_brain_region_container_mut(&mut self) -> &mut BrainRegionContainer<Self::BrainRegionContext, BrainRegionHierarchyContextRoot> {
        &mut self.root_brain_region
    }

    fn get_inner_brain_region_container(
        &self,
        brain_region_id: &BrainRegionID,
    ) -> Option<&BrainRegionContainer<Self::BrainRegionContext, BrainRegionHierarchyContextInner>> {
        self.inner_brain_regions.get(brain_region_id)
    }

    fn get_inner_brain_region_container_mut(
        &mut self,
        brain_region_id: &BrainRegionID,
    ) -> Option<&mut BrainRegionContainer<Self::BrainRegionContext, BrainRegionHierarchyContextInner>> {
        self.inner_brain_regions.get_mut(brain_region_id)
    }

    fn get_brain_region_ref(&self, brain_region_id: &BrainRegionID) -> Option<BrainRegionContainerRef<Self::BrainRegionContext>> {
        if let Some(region) = self.inner_brain_regions.get(brain_region_id) {
            return Some(BrainRegionContainerRef::Inner(region));
        }

        if brain_region_id == self.root_brain_region.get_root_id() {
            // check second since its only 1 region
            return Some(BrainRegionContainerRef::Root(&self.root_brain_region));
        }
        None
    }

    fn get_brain_region_ref_mut(&mut self, brain_region_id: &BrainRegionID) -> Option<BrainRegionContainerRefMut<Self::BrainRegionContext>> {
        if let Some(region) = self.inner_brain_regions.get_mut(brain_region_id) {
            return Some(BrainRegionContainerRefMut::Inner(region));
        }

        if brain_region_id == self.root_brain_region.get_root_id() {
            // check second since its only 1 region
            return Some(BrainRegionContainerRefMut::Root(&mut self.root_brain_region));
        }
        None
    }

    fn add_new_inner_brain_region(
        &mut self,
        ids: BrainRegionIdAndAncestryConstructor,
        parent_id: &BrainRegionID,
        metadata: GenomeObjectMetadata,
        positioning: GenomeObjectPositioning,
    ) -> Result<(), BrainRegionSetError> {
        let (id, ancestor_id) = ids.into_pair();
        if self.get_brain_region_ref(&id).is_some() {
            return Err(BrainRegionSetError::BrainRegionIDAlreadyExists);
        }
        if self.contains_ancestry_id(&ancestor_id) {
            return Err(BrainRegionSetError::BrainRegionAncestryIDAlreadyExists);
        }

        let parent = self
            .get_brain_region_ref_mut(parent_id)
            .ok_or_else(|| BrainRegionSetError::BrainRegionIDNotFound)?;

        match parent {
            BrainRegionContainerRefMut::Root(reg) => {
                if reg.context.contained_brain_regions.contains(&id) {
                    return Err(BrainRegionSetError::BrainRegionIDAlreadyExists);
                }
                reg.context.contained_brain_regions.insert(id);
            }
            BrainRegionContainerRefMut::Inner(reg) => {
                if reg.context.contained_brain_regions.contains(&id) {
                    return Err(BrainRegionSetError::BrainRegionIDAlreadyExists);
                }
                reg.context.contained_brain_regions.insert(id);
            }
        }

        let container = BrainRegionContainer::new_inner(ancestor_id, metadata, positioning, *parent_id);

        self.inner_brain_regions.insert(id, container);
        self.contained_ancestry_ids.insert(ancestor_id);
        Ok(())
    }

    fn remove_inner_brain_region(&mut self, container_id: &BrainRegionID) -> Result<(), BrainRegionSetError> {
        todo!()
    }

    fn move_inner_brain_region(
        &mut self,
        moving: Self::BrainRegionContext,
        from: &BrainRegionID,
        to: &BrainRegionID,
    ) -> Result<(), BrainRegionSetError> {
        todo!()
    }

    fn add_cortical_area(&mut self, cortical_id: CorticalIdentifier, parent_id: &BrainRegionID) -> Result<(), BrainRegionSetError> {
        // note: we check higher up that the cortical area already doesnt exist in general
        let parent = self
            .get_brain_region_ref_mut(parent_id)
            .ok_or_else(|| BrainRegionSetError::BrainRegionIDNotFound)?;

        match parent {
            BrainRegionContainerRefMut::Root(reg) => {
                if reg.context.contained_cortical_areas.contains(&cortical_id) {
                    return Err(BrainRegionSetError::CorticalIDAlreadyExists);
                }
                reg.context.contained_cortical_areas.insert(cortical_id);
            }
            BrainRegionContainerRefMut::Inner(reg) => {
                if reg.context.contained_cortical_areas.contains(&cortical_id) {
                    return Err(BrainRegionSetError::CorticalIDAlreadyExists);
                }
                reg.context.contained_cortical_areas.insert(cortical_id);
            }
        }

        Ok(())
    }

    fn move_cortical_area(&mut self, moving: Self::BrainRegionContext, from: &BrainRegionID, to: &BrainRegionID) -> Result<(), BrainRegionSetError> {
        todo!()
    }
}

//region Containers

//region Contexts

//region Genome Context

pub trait BrainRegionGenomeContext: Default {}

//region Storage

/// In the case of dense storage, we dont want to store additional data
#[derive(Default, Clone)]
pub struct BrainRegionContextStorage;

impl BrainRegionGenomeContext for BrainRegionContextStorage {}

//endregion

/// In the case of runtime, we will use memory to offer rapid lookups
#[derive(Default)]
pub struct BrainRegionContextDesigner {
    pub contained_brain_regions: HashSet<BrainRegionID>,
    pub contained_cortical_areas: HashSet<CorticalIdentifier>,
}

impl BrainRegionGenomeContext for BrainRegionContextDesigner {}

//endregion

//region Hierarchy Context

pub trait BrainRegionHierarchyContext: Clone {
    const IS_ROOT: bool;
}

#[derive(Clone)]
pub struct BrainRegionHierarchyContextRoot {
    pub root_id: BrainRegionID,
}

impl BrainRegionHierarchyContext for BrainRegionHierarchyContextRoot {
    const IS_ROOT: bool = true;
}

#[derive(Clone)]
pub struct BrainRegionHierarchyContextInner {
    pub parent_region: BrainRegionID,
    pub positioning: GenomeObjectPositioning,
}

impl BrainRegionHierarchyContext for BrainRegionHierarchyContextInner {
    const IS_ROOT: bool = false;
}

//endregion

//endregion

//region Data

/// Methods expected for any container type, all brain regions will have these immut methods
pub trait BrainRegionContainerUniversal {
    fn get_ancestor_id(&self) -> &BrainRegionAncestryID;
    fn get_metadata(&self) -> &GenomeObjectMetadata;
    fn is_root_region(&self) -> bool;
}

/// Methods expected for any mut container type, all brain regions will have these mut methods
pub trait BrainRegionContainerUniversalMut: BrainRegionContainerUniversal {
    fn get_metadata_mut(&mut self) -> &mut GenomeObjectMetadata;
}

//region Container

#[derive(Clone)]
pub struct BrainRegionContainer<BrainRegionContext: BrainRegionGenomeContext, HierarchyContext: BrainRegionHierarchyContext> {
    ancestor_id: BrainRegionAncestryID,
    metadata: GenomeObjectMetadata,
    hierarchy: HierarchyContext,
    context: BrainRegionContext,
}

impl<CachingContext: BrainRegionGenomeContext, HierarchyContext: BrainRegionHierarchyContext> BrainRegionContainerUniversal
    for BrainRegionContainer<CachingContext, HierarchyContext>
{
    fn get_ancestor_id(&self) -> &BrainRegionAncestryID {
        &self.ancestor_id
    }
    fn get_metadata(&self) -> &GenomeObjectMetadata {
        &self.metadata
    }
    fn is_root_region(&self) -> bool { HierarchyContext::IS_ROOT }
}

impl<CachingContext: BrainRegionGenomeContext, HierarchyContext: BrainRegionHierarchyContext> BrainRegionContainerUniversalMut
    for BrainRegionContainer<CachingContext, HierarchyContext>
{
    fn get_metadata_mut(&mut self) -> &mut GenomeObjectMetadata {
        &mut self.metadata
    }
}

impl<CachingContext: BrainRegionGenomeContext> BrainRegionContainer<CachingContext, BrainRegionHierarchyContextRoot> {
    pub fn new_root(ancestor_id: BrainRegionAncestryID, metadata: GenomeObjectMetadata, root_id: BrainRegionID) -> Self {
        Self {
            ancestor_id,
            metadata,
            hierarchy: BrainRegionHierarchyContextRoot { root_id },
            context: Default::default(),
        }
    }

    pub fn get_root_id(&self) -> &BrainRegionID {
        &self.hierarchy.root_id
    }
}

impl<CachingContext: BrainRegionGenomeContext> BrainRegionContainer<CachingContext, BrainRegionHierarchyContextInner> {
    pub fn new_inner(
        ancestor_id: BrainRegionAncestryID,
        metadata: GenomeObjectMetadata,
        positioning: GenomeObjectPositioning,
        parent_region: BrainRegionID,
    ) -> Self {
        Self {
            ancestor_id,
            metadata,
            hierarchy: BrainRegionHierarchyContextInner { parent_region, positioning },
            context: Default::default(),
        }
    }

    pub fn get_positioning(&self) -> &GenomeObjectPositioning {
        &self.hierarchy.positioning
    }

    pub fn get_positioning_mut(&mut self) -> &mut GenomeObjectPositioning {
        &mut self.hierarchy.positioning
    }
}

//endregion

//region ContainerRef / Mut

pub enum BrainRegionContainerRef<'a, CachingContext: BrainRegionGenomeContext> {
    Root(&'a BrainRegionContainer<CachingContext, BrainRegionHierarchyContextRoot>),
    Inner(&'a BrainRegionContainer<CachingContext, BrainRegionHierarchyContextInner>),
}

impl<'a, CachingContext: BrainRegionGenomeContext> BrainRegionContainerUniversal for BrainRegionContainerRef<'a, CachingContext> {
    fn get_ancestor_id(&self) -> &BrainRegionAncestryID {
        match self {
            BrainRegionContainerRef::Root(region) => region.get_ancestor_id(),
            BrainRegionContainerRef::Inner(region) => region.get_ancestor_id(),
        }
    }

    fn get_metadata(&self) -> &GenomeObjectMetadata {
        match self {
            BrainRegionContainerRef::Root(region) => region.get_metadata(),
            BrainRegionContainerRef::Inner(region) => region.get_metadata(),
        }
    }

    fn is_root_region(&self) -> bool {
        match self {
            BrainRegionContainerRef::Root(_) => {true}
            BrainRegionContainerRef::Inner(_) => {false}
        }
    }
}

pub enum BrainRegionContainerRefMut<'a, CachingContext: BrainRegionGenomeContext> {
    Root(&'a mut BrainRegionContainer<CachingContext, BrainRegionHierarchyContextRoot>),
    Inner(&'a mut BrainRegionContainer<CachingContext, BrainRegionHierarchyContextInner>),
}

impl<'a, CachingContext: BrainRegionGenomeContext> BrainRegionContainerUniversal for BrainRegionContainerRefMut<'a, CachingContext> {
    fn get_ancestor_id(&self) -> &BrainRegionAncestryID {
        match self {
            BrainRegionContainerRefMut::Root(region) => region.get_ancestor_id(),
            BrainRegionContainerRefMut::Inner(region) => region.get_ancestor_id(),
        }
    }

    fn get_metadata(&self) -> &GenomeObjectMetadata {
        match self {
            BrainRegionContainerRefMut::Root(region) => region.get_metadata(),
            BrainRegionContainerRefMut::Inner(region) => region.get_metadata(),
        }
    }

    fn is_root_region(&self) -> bool {
        match self {
            BrainRegionContainerRefMut::Root(_) => {true}
            BrainRegionContainerRefMut::Inner(_) => {false}
        }
    }
}

impl<'a, CachingContext: BrainRegionGenomeContext> BrainRegionContainerUniversalMut for BrainRegionContainerRefMut<'a, CachingContext> {
    fn get_metadata_mut(&mut self) -> &mut GenomeObjectMetadata {
        match self {
            BrainRegionContainerRefMut::Root(region) => region.get_metadata_mut(),
            BrainRegionContainerRefMut::Inner(region) => region.get_metadata_mut(),
        }
    }
}
//endregion

//endregion

//endregion

#[derive(Debug, thiserror::Error)]
pub enum BrainRegionSetError {
    #[error("The given brain region could not be found")]
    BrainRegionIDNotFound,
    #[error("The given inner brain region could not be found")]
    InnerBrainRegionIDNotFound,
    #[error("The given brain region id already exists")]
    BrainRegionIDAlreadyExists,
    #[error("The given brain region ancestry id already exists")]
    BrainRegionAncestryIDAlreadyExists,
    #[error("The given cortical area could not be found within the given brain region")]
    CorticalIDNotFound,
    #[error("The given cortical area already exists within the given brain region")]
    CorticalIDAlreadyExists,
}
