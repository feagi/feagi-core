use crate::basis::brain_region::BrainRegionID;
use crate::basis::cortical_area::foundation::CTGeneratorFoundationPowerModeled;
use crate::basis::cortical_area::processing::CTGeneratorProcessingInterconnectModeled;
use crate::basis::cortical_area::CorticalIdentifier;
use crate::genome::genome_object::GenomeObjectMetadata;
use crate::genome::genome_object::GenomeObjectPositioning;
use feagi_connectome::feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct CorticalAreaGenomeCorticalTypedMap<FIQ: FeagiIndexQuantization>
{
    pub foundation: CorticalAreaGenomeCorticalFoundationSubtypedMap<FIQ>,
    pub processing: CorticalAreaGenomeCorticalProcessingSubtypedMap<FIQ>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct CorticalAreaGenomeCorticalFoundationSubtypedMap<FIQ: FeagiIndexQuantization>
{
    pub power: HashMap<[u8; 6], CorticalAreaContainerCommon<CTGeneratorFoundationPowerModeled<FIQ>>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct CorticalAreaGenomeCorticalProcessingSubtypedMap<FIQ: FeagiIndexQuantization>
{
    pub interconnect: HashMap<[u8; 6], CorticalAreaContainerCommon<CTGeneratorProcessingInterconnectModeled<FIQ>>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CorticalAreaContainerCommon<CorticalGeneratorEnum> {
    pub metadata: GenomeObjectMetadata,
    pub positioning: GenomeObjectPositioning,
    pub parent_id: BrainRegionID,
    pub incoming_mappings: HashSet<CorticalIdentifier>,
    pub outgoing_mappings: HashSet<CorticalIdentifier>,
    pub genome_generator: CorticalGeneratorEnum // As long as its not a storage type, this needs to be small
}





