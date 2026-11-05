use crate::cortical_area::CorticalAreaTypeMap;
use crate::basis::specifications::brain_regions::BrainRegionID;
use feagi_connectome::quantization_levels::FeagiIndexQuantization;
use std::collections::HashMap;
use crate::genome::containers::brain_region::BrainRegionContainerRuntime;
use crate::genome::containers::cortical_area::CorticalAreaContainersDesigner;

/// A dynamic Genome that stores genomic level data for the running feagi brain and acts
/// as the interface and translator for modifications to that brain directly on the genome and
/// through the NPU
pub struct DesignerGenome<FIQ: FeagiIndexQuantization> {
    genome_name: String,
    root_brain_region_id: BrainRegionID,
    brain_regions: HashMap<BrainRegionID, BrainRegionContainerRuntime>,
    cortical_areas: CorticalAreaTypeMap<CorticalAreaContainersDesigner<FIQ>>,

    // TODO NPU

    // TODO
    // connectivity rules (id -> rule)
    // cortical mappings (id -> cortical mapping set (mapping id -> mapping) )
    _p: core::marker::PhantomData<FIQ>,
}

impl<FIQ: FeagiIndexQuantization> DesignerGenome<FIQ> {
    
    
    pub fn new_genome_now(genome_name: String) -> Self {
        Self {
            genome_name,
            root_brain_region_id: BrainRegionID::new_now(),
            brain_regions: Default::default(),
            cortical_areas: Default::default(),
            _p: Default::default(),
        }
    }
}
