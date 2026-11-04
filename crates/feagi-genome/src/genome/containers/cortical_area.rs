use crate::basis::specifications::brain_regions::BrainRegionID;
use crate::cortical_area::CorticalIdentifier;
use crate::genome::genome_object::GenomeObjectMetadata;
use crate::genome::genome_object::GenomeObjectPositioning;
use feagi_connectome::cortical_area::NeuronModel;
use feagi_connectome::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use std::collections::HashSet;


pub struct CorticalAreaContainerTypedDesigner<FIQ: FeagiIndexQuantization, NM: NeuronModel>
{
    pub metadata: GenomeObjectMetadata,
    pub positioning: GenomeObjectPositioning,
    pub parent_id: BrainRegionID,
    pub cortical_area_model: (),
    pub incoming_mappings: HashSet<CorticalIdentifier>,
    pub outgoing_mappings: HashSet<CorticalIdentifier>,
}


/*

pub struct CorticalAreaContainerTypedStorage<FIQ: FeagiIndexQuantization, NM: NeuronModel>
{
    pub metadata: GenomeObjectMetadata,
    pub positioning: GenomeObjectPositioning,
    pub parent_id: BrainRegionID,
    pub cortical_area_model: (),
}

 */



