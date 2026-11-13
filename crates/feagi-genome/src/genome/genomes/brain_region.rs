use std::collections::{HashMap, HashSet};
use crate::basis::brain_region::{BrainRegionAncestryID, BrainRegionID};
use crate::basis::cortical_area::cortical_identifiers::CorticalIdentifier;
use crate::genome::genome_object::{GenomeObjectMetadata, GenomeObjectPositioning};
pub struct BrainRegionSet<CachingContext: BrainRegionCachingContext> {
    root_brain_region: BrainRegionContainer<CachingContext, BrainRegionHierarchyContextRoot>,
    inner_brain_regions: HashMap<BrainRegionID, BrainRegionContainer<CachingContext, BrainRegionHierarchyContextInner>>,
}

impl<CachingContext: BrainRegionCachingContext> BrainRegionSet<CachingContext> {

    pub fn new_brain_region_set(
        metadata: GenomeObjectMetadata,
        root_id: BrainRegionID,
        ancestor_id: BrainRegionAncestryID
    ) -> Self {
        let root_brain_region = BrainRegionContainer::new_root(ancestor_id, metadata, root_id);
        Self {
            root_brain_region,
            inner_brain_regions: Default::default()
        }
    }
    
    pub fn get_root_brain_region_container(&self) -> &BrainRegionContainer<CachingContext, BrainRegionHierarchyContextRoot> {
        &self.root_brain_region
    }

    pub fn get_root_brain_region_container_mut(&mut self) -> &mut BrainRegionContainer<CachingContext, BrainRegionHierarchyContextRoot> {
        &mut self.root_brain_region
    }
    
    pub fn get_inner_brain_region_container(&self, brain_region_id: &BrainRegionID) -> Option<&BrainRegionContainer<CachingContext, BrainRegionHierarchyContextInner>> {
        self.inner_brain_regions.get(brain_region_id)
    }

    pub fn get_inner_brain_region_container_mut(&mut self, brain_region_id: &BrainRegionID) -> Option<&mut BrainRegionContainer<CachingContext, BrainRegionHierarchyContextInner>> {
        self.inner_brain_regions.get_mut(brain_region_id)
    }
    
    pub fn get_brain_region_ref(&self, brain_region_id: &BrainRegionID) -> Option<BrainRegionContainerRef<CachingContext>> {
        
        if let Some(region) = self.inner_brain_regions.get(brain_region_id) {
            return Some(BrainRegionContainerRef::Inner(region))
        }

        if brain_region_id == self.root_brain_region.get_root_id() { // check second since its only 1 region
            return Some(BrainRegionContainerRef::Root(&self.root_brain_region))
        }
        None

    }

    pub fn get_brain_region_ref_mut(&mut self, brain_region_id: &BrainRegionID) -> Option<BrainRegionContainerRefMut<CachingContext>> {

        if let Some(region) = self.inner_brain_regions.get_mut(brain_region_id) {
            return Some(BrainRegionContainerRefMut::Inner(region))
        }

        if brain_region_id == self.root_brain_region.get_root_id() { // check second since its only 1 region
            return Some(BrainRegionContainerRefMut::Root(&mut self.root_brain_region))
        }
        None

    }

    // NOTE: Should only be called by `BrainRegionCachingContext`
    pub fn add_new_brain_region(&mut self, new_container_id: BrainRegionID, new_container: BrainRegionContainer<CachingContext, BrainRegionHierarchyContextInner>) {
        // TODO maybe add collision checks, but is that really that likely with uuidv7?
        self.inner_brain_regions.insert(new_container_id, new_container);
    }
}

// TODO Iter


//region Contexts

pub trait BrainRegionCachingContext: Default {
    
    fn add_new_now_region(
        brain_region_set: &mut BrainRegionSet<Self>,
        metadata: GenomeObjectMetadata,
        positioning: GenomeObjectPositioning,
    ) {
        let region_id = BrainRegionID::new_now();
        let ancestor_id = region_id.clone().into();
        
        let region = BrainRegionContainer::new_inner(
            ancestor_id,
            metadata,
            positioning
        );

        brain_region_set.add_new_brain_region(region_id, region);
    }
    
}

/// In the case of dense storage, we dont want to store additional data
#[derive(Default, Clone)]
pub struct BrainRegionCachingContextNone;

impl BrainRegionCachingContext for BrainRegionCachingContextNone {}

/// In the case of runtime, we will use memory to offer rapid lookups
#[derive(Default)]
pub struct BrainRegionCachingContextCaching {
    pub contained_brain_regions: HashSet<BrainRegionID>,
    pub contained_cortical_areas: HashSet<CorticalIdentifier>,
}

impl BrainRegionCachingContext for BrainRegionCachingContextCaching {}



pub trait BrainRegionHierarchyContext: Clone {}

#[derive(Clone)]
pub struct BrainRegionHierarchyContextRoot {
    pub root_id: BrainRegionID,
}

impl BrainRegionHierarchyContext for BrainRegionHierarchyContextRoot {}

#[derive(Clone)]
pub struct BrainRegionHierarchyContextInner {
    pub positioning: GenomeObjectPositioning,
}

impl BrainRegionHierarchyContext for BrainRegionHierarchyContextInner {}

//endregion

//region Data

/// Methods expected for any container type, all brain regions will have these immut methods
pub trait BrainRegionContainerUniversal {
    fn get_ancestor_id(&self) -> &BrainRegionAncestryID;
    fn get_metadata(&self) -> &GenomeObjectMetadata;
}

/// Methods expected for any mut container type, all brain regions will have these mut methods
pub trait BrainRegionContainerUniversalMut: BrainRegionContainerUniversal {
    fn get_metadata_mut(&mut self) -> &mut GenomeObjectMetadata;
}

//region Container

#[derive(Clone)]
pub struct BrainRegionContainer<
    CachingContext: BrainRegionCachingContext,
    HierarchyContext: BrainRegionHierarchyContext

> {
    ancestor_id: BrainRegionAncestryID,
    metadata: GenomeObjectMetadata,
    hierarchy: HierarchyContext,
    cache: CachingContext
}

impl<
    CachingContext: BrainRegionCachingContext,
    HierarchyContext: BrainRegionHierarchyContext
> BrainRegionContainerUniversal for BrainRegionContainer<CachingContext, HierarchyContext> {
    fn get_ancestor_id(&self) -> &BrainRegionAncestryID {
        &self.ancestor_id
    }
    fn get_metadata(&self) -> &GenomeObjectMetadata {
        &self.metadata
    }
}

impl<
    CachingContext: BrainRegionCachingContext,
    HierarchyContext: BrainRegionHierarchyContext
> BrainRegionContainerUniversalMut for BrainRegionContainer<CachingContext, HierarchyContext> {
    fn get_metadata_mut(&mut self) -> &mut GenomeObjectMetadata {
        &mut self.metadata
    }
}

impl<CachingContext: BrainRegionCachingContext>
BrainRegionContainer<CachingContext, BrainRegionHierarchyContextRoot>
{
    pub fn new_root(ancestor_id: BrainRegionAncestryID, metadata: GenomeObjectMetadata, root_id: BrainRegionID, ) -> Self {
        Self {
            ancestor_id,
            metadata,
            hierarchy: BrainRegionHierarchyContextRoot { root_id },
            cache: Default::default(),
        }
    }

    pub fn get_root_id(&self) -> &BrainRegionID {
        &self.hierarchy.root_id
    }
}

impl<CachingContext: BrainRegionCachingContext>
BrainRegionContainer<CachingContext, BrainRegionHierarchyContextInner>
{
    pub fn new_inner(ancestor_id: BrainRegionAncestryID, metadata: GenomeObjectMetadata, positioning: GenomeObjectPositioning, ) -> Self {
        Self {
            ancestor_id,
            metadata,
            hierarchy: BrainRegionHierarchyContextInner { positioning },
            cache: Default::default(),
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


pub enum BrainRegionContainerRef<'a, CachingContext: BrainRegionCachingContext> {
    Root(&'a BrainRegionContainer<CachingContext, BrainRegionHierarchyContextRoot>),
    Inner(&'a BrainRegionContainer<CachingContext, BrainRegionHierarchyContextInner>)
}

impl<'a, CachingContext: BrainRegionCachingContext> BrainRegionContainerUniversal
for BrainRegionContainerRef<'a, CachingContext> {
    fn get_ancestor_id(&self) -> &BrainRegionAncestryID {
        match self {
            BrainRegionContainerRef::Root(region) => {region.get_ancestor_id()}
            BrainRegionContainerRef::Inner(region) => {region.get_ancestor_id()}
        }
    }

    fn get_metadata(&self) -> &GenomeObjectMetadata {
        match self {
            BrainRegionContainerRef::Root(region) => {region.get_metadata()}
            BrainRegionContainerRef::Inner(region) => {region.get_metadata()}
        }
    }
}


pub enum BrainRegionContainerRefMut<'a, CachingContext: BrainRegionCachingContext> {
    Root(&'a mut BrainRegionContainer<CachingContext, BrainRegionHierarchyContextRoot>),
    Inner(&'a mut BrainRegionContainer<CachingContext, BrainRegionHierarchyContextInner>)
}

impl<'a, CachingContext: BrainRegionCachingContext> BrainRegionContainerUniversal
for BrainRegionContainerRefMut<'a, CachingContext> {
    fn get_ancestor_id(&self) -> &BrainRegionAncestryID {
        match self {
            BrainRegionContainerRefMut::Root(region) => {region.get_ancestor_id()}
            BrainRegionContainerRefMut::Inner(region) => {region.get_ancestor_id()}
        }
    }

    fn get_metadata(&self) -> &GenomeObjectMetadata {
        match self {
            BrainRegionContainerRefMut::Root(region) => {region.get_metadata()}
            BrainRegionContainerRefMut::Inner(region) => {region.get_metadata()}
        }
    }
}

impl<'a, CachingContext: BrainRegionCachingContext> BrainRegionContainerUniversalMut for BrainRegionContainerRefMut<'a, CachingContext> {
    fn get_metadata_mut(&mut self) -> &mut GenomeObjectMetadata {
        match self {
            BrainRegionContainerRefMut::Root(region) => { region.get_metadata_mut() }
            BrainRegionContainerRefMut::Inner(region) => { region.get_metadata_mut() }
        }
    }
}
//endregion

//endregion
