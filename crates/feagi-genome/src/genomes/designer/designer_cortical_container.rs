use std::collections::HashSet;
use crate::basis::cortical_area::cortical_id::CorticalID;
use crate::basis::genome_object_metadata::GenomeObjectMetadata;
use crate::identifiers::brain_regions::BrainRegionID;
use feagi_connectome::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_connectome::neuron_models::enums::NeuronModelGenomePropertiesEnum;

pub struct DesignerCorticalAreaContainer<FIQ: FeagiIndexQuantization> {
    metadata: GenomeObjectMetadata,
    model_data: NeuronModelGenomePropertiesEnum<FIQ>,
    cache_parent_region_id: BrainRegionID,
    cache_incoming_mappings: HashSet<CorticalID>,
    cache_outgoing_mappings: HashSet<CorticalID>,
}

impl<FIQ: FeagiIndexQuantization> DesignerCorticalAreaContainer<FIQ> {

    pub fn new(
        metadata: GenomeObjectMetadata,
        model_data: NeuronModelGenomePropertiesEnum<FIQ>,
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
