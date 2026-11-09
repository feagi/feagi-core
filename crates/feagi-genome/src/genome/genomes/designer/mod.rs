pub mod container_brain_region;

use crate::basis::cortical_area::CorticalAreaTypeMap;
use crate::basis::brain_region_identifiers::BrainRegionID;
use std::collections::HashMap;
use feagi_connectome::connectome_basis::quantization_levels::FeagiIndexQuantization;
use container_brain_region::BrainRegionContainerRuntime;
use crate::genome::cortical_area::CorticalAreaContainersDesigner;
use crate::genome::genome_object::{GenomeObjectMetadata, GenomeObjectPositioning};
use crate::genome::genomes::designer::container_brain_region::{BrainRegionContainerInnerRuntime, BrainRegionContainerRootRuntime};
use crate::genome::genomes::GenomeError;

/// A dynamic Genome that stores genomic level data for the running feagi brain and acts
/// as the interface and translator for modifications to that brain directly on the genome and
/// through the NPU
pub struct DesignerGenome<FIQ: FeagiIndexQuantization> {
    root_brain_region: BrainRegionContainerRootRuntime,
    inner_brain_regions: HashMap<BrainRegionID, BrainRegionContainerInnerRuntime>,
    cortical_areas: CorticalAreaTypeMap<CorticalAreaContainersDesigner<FIQ>>,

    // TODO NPU

    // TODO
    // connectivity rules (id -> rule)
    // cortical mappings (id -> cortical mapping set (mapping id -> mapping) )
    _p: core::marker::PhantomData<FIQ>,
}

impl<FIQ: FeagiIndexQuantization> DesignerGenome<FIQ> {

    pub fn new_genome_now(root_metadata: GenomeObjectMetadata) -> Self {

        let (root, _root_id)
            = BrainRegionContainerRootRuntime::new_now(root_metadata);
        Self {
            root_brain_region: root,
            inner_brain_regions: Default::default(),
            cortical_areas: Default::default(),
            _p: Default::default(),
        }
    }

    pub fn get_brain_region(&self, id: &BrainRegionID) -> Result<&BrainRegionContainerRuntime, GenomeError> {
        if &self.root_brain_region.root_id == id {
            return Ok(&BrainRegionContainerRuntime::Root(self.root_brain_region))
        }
        if let Some(brain_region) = self.inner_brain_regions.get(&id) {
            return Ok(&BrainRegionContainerRuntime::Inner(*brain_region))
        }
        Err(GenomeError::InvalidParentRegionID)
    }

    pub(crate) fn get_brain_region_mut(&mut self, id: &BrainRegionID) -> Result<&mut BrainRegionContainerRuntime, GenomeError> {
        if &self.root_brain_region.root_id == id {
            return Ok(&mut BrainRegionContainerRuntime::Root(self.root_brain_region))
        }
        if let Some(brain_region) = self.inner_brain_regions.get(&id) {
            return Ok(&mut BrainRegionContainerRuntime::Inner(*brain_region))
        }
        Err(GenomeError::InvalidParentRegionID)
    }



    pub fn add_new_brain_region(&mut self, parent_region_id: &BrainRegionID, metadata: GenomeObjectMetadata, positioning: GenomeObjectPositioning) -> Result<(), GenomeError> {
        let parent_brain_region = self.get_brain_region_mut(parent_region_id)?;
        let (new_region, new_id) = BrainRegionContainerInnerRuntime::new_now(parent_region_id.clone(), metadata, positioning);
        parent_brain_region.insert_contained_brain_region_id(new_id);
        self.inner_brain_regions.insert(new_id, new_region);
        Ok(())
    }



}
