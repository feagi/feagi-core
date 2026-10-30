use std::collections::HashSet;
use feagi_connectome::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::basis::cortical_area::cortical_id::CorticalID;
use crate::cortical_area_types::enums::CorticalGenomicDataEnum;
use crate::genomes::genome_object_metadata::GenomeObjectMetadata;
use crate::identifiers::brain_regions::BrainRegionID;
use crate::genomes::genome_object_positioning::GenomeObjectPositioning;

pub struct DesignerCorticalAreaContainer<FIQ: FeagiIndexQuantization> {
    metadata: GenomeObjectMetadata,
    positioning: GenomeObjectPositioning,
    data: CorticalGenomicDataEnum<FIQ>, // TODO not this one actually

    // Cache items
    cache_parent_region_id: BrainRegionID,
    cache_incoming_mappings: HashSet<CorticalID>,
    cache_outgoing_mappings: HashSet<CorticalID>,
}

impl<FIQ: FeagiIndexQuantization> DesignerCorticalAreaContainer<FIQ> {

    pub fn new(
        metadata: GenomeObjectMetadata,
        positioning: GenomeObjectPositioning,
        data: CorticalGenomicDataEnum<FIQ>,
        cache_parent_region_id: BrainRegionID,
        cache_incoming_mappings: HashSet<CorticalID>,
        cache_outgoing_mappings: HashSet<CorticalID>,
    ) -> Self {
        Self {
            metadata,
            positioning,
            data,
            cache_parent_region_id,
            cache_incoming_mappings,
            cache_outgoing_mappings,
        }
    }
}

// Exposed Getters
impl<FIQ: FeagiIndexQuantization> DesignerCorticalAreaContainer<FIQ> {
    /// User editable Cortical Name and Description
    pub fn metadata(&self) -> &GenomeObjectMetadata {
        &self.metadata
    }

    /// Positioning relative to the parent region
    pub fn positioning(&self) -> GenomeObjectPositioning {
        self.positioning
    }

    pub fn data(&self) -> &CorticalGenomicDataEnum<FIQ> {
        &self.data
    }

    pub fn parent_region_id(&self) -> BrainRegionID {
        self.cache_parent_region_id
    }

    pub fn incoming_cortical_ids(&self) -> &HashSet<CorticalID> {
        &self.cache_incoming_mappings
    }

    pub fn outgoing_cortical_ids(&self) -> &HashSet<CorticalID> {
        &self.cache_outgoing_mappings
    }
}