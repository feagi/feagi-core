use crate::basis::cortical_area::cortical_id_legacy::CorticalIDLegacy;
use crate::basis::specifications::brain_regions::BrainRegionID;
use crate::basis::specifications::genome_id::GenomeID;
use crate::genome::genomes::designer::designer_brain_region::BrainRegionDesignerContainer;
use crate::genome::genomes::designer::designer_cortical_container::DesignerCorticalAreaContainer;
use feagi_connectome::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use std::collections::HashMap;

/// A dynamic Genome that stores genomic level data for the running feagi brain and acts
/// as the interface and translator for modifications to that brain directly on the genome and
/// through the NPU
pub struct DesignerGenome<FIQ: FeagiIndexQuantization> {
    genome_name: String,
    genome_id: GenomeID,
    brain_regions: HashMap<BrainRegionID, BrainRegionDesignerContainer>,
    cortical_areas: HashMap<CorticalIDLegacy, DesignerCorticalAreaContainer<FIQ>>,

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
            genome_id: GenomeID::new_now(),
            brain_regions: Default::default(),
            cortical_areas: Default::default(),
            _p: Default::default(),
        }
    }
}
