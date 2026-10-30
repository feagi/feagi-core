use std::collections::HashMap;
use feagi_connectome::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::basis::cortical_area::cortical_id::CorticalID;
use crate::genomes::designer::designer_cortical_container::DesignerCorticalAreaContainer;
use crate::identifiers::brain_regions::BrainRegionID;

/// A dynamic Genome that stores genomic level data for the running feagi brain and acts
/// as the interface and translator for modifications to that brain directly on the genome and
/// through the NPU
pub struct DesignerGenome<FIQ: FeagiIndexQuantization> {
    genome_name: String,
    root_brain_region: (BrainRegionID, ()),
    brain_regions: HashMap<BrainRegionID, ()>,
    cortical_areas: HashMap<CorticalID, DesignerCorticalAreaContainer<FIQ>>,

    // TODO NPU


    // TODO
    // connectivity rules (id -> rule)
    // cortical mappings (id -> cortical mapping set (mapping id -> mapping) )

    _p: core::marker::PhantomData<FIQ>,
}