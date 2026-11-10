pub mod container_brain_region;
pub mod container_cortical_area;

use container_brain_region::{BrainRegionContainerRuntimeEnumMut, BrainRegionContainerRuntimeEnumRef};
use std::collections::HashMap;
use crate::basis::brain_region::BrainRegionID;
use crate::genome::genome_object::{GenomeObjectMetadata, GenomeObjectPositioning};
use crate::genome::genomes::designer::container_brain_region::{BrainRegionContainerInnerRuntime, BrainRegionContainerRootRuntime};
use crate::genome::genomes::GenomeError;
use feagi_connectome::feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
use crate::genome::genomes::designer::container_cortical_area::CorticalAreaGenomeCorticalTypedMap;

/// A dynamic Genome that stores genomic level data for the running feagi brain and acts
/// as the interface and translator for modifications to that brain directly on the genome and
/// through the NPU
pub struct DesignerGenome<FIQ: FeagiIndexQuantization> {
    root_brain_region: BrainRegionContainerRootRuntime,
    inner_brain_regions: HashMap<BrainRegionID, BrainRegionContainerInnerRuntime>,
    cortical_areas: CorticalAreaGenomeCorticalTypedMap<FIQ>,

    // TODO NPU

    // TODO
    // connectivity rules cache (id -> rule)
    // cortical mappings cache  (id -> cortical mapping set (mapping id -> mapping) )
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

    pub fn get_brain_region(&self, id: &BrainRegionID) -> Result<BrainRegionContainerRuntimeEnumRef<'_>, GenomeError> {
        if &self.root_brain_region.root_id == id {
            return Ok(BrainRegionContainerRuntimeEnumRef::Root(&self.root_brain_region))
        }
        if let Some(brain_region) = self.inner_brain_regions.get(id) {
            return Ok(BrainRegionContainerRuntimeEnumRef::Inner(brain_region))
        }
        Err(GenomeError::InvalidRegionID)
    }

    // Some methods, while mutables, will have no dependents and thus can be executed safely independently
    //region Safe Mutable methods


    pub fn reposition_brain_region(&mut self, region_id: &BrainRegionID, new_pos: GenomeObjectPositioning) -> Result<(), GenomeError> {
        let mut region_enum = self.get_brain_region_mut(region_id)?;
        let region = region_enum.try_unwrap_as_inner_mut()?;
        region.positioning = new_pos;
        Ok(())
    }

    //endregion


    pub fn add_new_brain_region(&mut self, parent_region_id: &BrainRegionID, metadata: GenomeObjectMetadata, positioning: GenomeObjectPositioning) -> Result<(), GenomeError> {
        let (new_region, new_id) = BrainRegionContainerInnerRuntime::new_now(parent_region_id.clone(), metadata, positioning);
        self.get_brain_region_mut(parent_region_id)?.insert_contained_brain_region_id(new_id.clone());
        self.inner_brain_regions.insert(new_id, new_region);
        Ok(())
    }



    pub(crate) fn get_brain_region_mut(&mut self, id: &BrainRegionID) -> Result<BrainRegionContainerRuntimeEnumMut<'_>, GenomeError> {
        if &self.root_brain_region.root_id == id {
            return Ok(BrainRegionContainerRuntimeEnumMut::Root(&mut self.root_brain_region))
        }
        if let Some(brain_region) = self.inner_brain_regions.get_mut(id) {
            return Ok(BrainRegionContainerRuntimeEnumMut::Inner(brain_region))
        }
        Err(GenomeError::InvalidRegionID)
    }










}
