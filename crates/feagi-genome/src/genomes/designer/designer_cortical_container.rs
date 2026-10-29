use crate::basis::cortical_area::cortical_id::CorticalID;
use crate::basis::genome_object_metadata::GenomeObjectMetadata;
use crate::identifiers::brain_regions::BrainRegionID;
use feagi_connectome::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use std::collections::HashSet;

pub struct DesignerCorticalContainer<FIQ: FeagiIndexQuantization> {
    metadata: GenomeObjectMetadata,
    model_data: (),
    cache_parent_region_id: BrainRegionID,
    cache_incoming_mappings: HashSet<CorticalID>,
    cache_outgoing_mappings: HashSet<CorticalID>,
}

impl<FIQ: FeagiIndexQuantization> DesignerCorticalContainer<FIQ> {

    pub fn new(
        metadata: GenomeObjectMetadata,
        model_data: (),
        cache_parent_region_id: BrainRegionID,
        cache_incoming_mappings: HashSet<CorticalID>,
        cache_outgoing_mappings: HashSet<CorticalID>,
    ) -> Self {
        Self {
            metadata,
            model_data,
            cache_parent_region_id,
            cache_incoming_mappings,
            cache_outgoing_mappings,
        }
    }
}
