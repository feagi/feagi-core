use crate::basis::brain_region::BrainRegionID;
use crate::basis::cortical_area::cortical_identifiers::CorticalIdentifier;
use crate::genome::genome_object::GenomeObjectMetadata;
use crate::genome::genome_object::GenomeObjectPositioning;
use feagi_connectome::feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
use std::collections::{HashSet};
use serde::{Deserialize, Serialize};
use crate::basis::cortical_area::CTGeneratorTyped;


#[derive(Debug, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct CorticalAreaContainer<FIQ: FeagiIndexQuantization> {
    pub metadata: GenomeObjectMetadata,
    pub positioning: GenomeObjectPositioning,
    pub parent_id: BrainRegionID,
    pub incoming_mappings: HashSet<CorticalIdentifier>,
    pub outgoing_mappings: HashSet<CorticalIdentifier>,
    pub genome_generator: CTGeneratorTyped<FIQ> // As long as its not a storage type, this needs to be small
}








