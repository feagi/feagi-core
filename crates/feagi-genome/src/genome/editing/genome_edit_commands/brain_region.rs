use serde::{Deserialize, Serialize};
use feagi_connectome::connectome_basis::quantization_levels::FeagiIndexQuantization;
use crate::basis::brain_region::{BrainRegionID};
use crate::genome::editing::genome_edit_commands::{GenomeEditCommandStep, GenomeEditCommandStepEnum};
use crate::genome::genome_object::{GenomeObjectMetadata, GenomeObjectPositioning};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenomeEditCommandStepBrainRegionAdd {
    pub metadata: GenomeObjectMetadata,
    pub positioning: GenomeObjectPositioning,
    pub this_id: BrainRegionID,
    pub parent_region_id: BrainRegionID,
}

impl<FIQ: FeagiIndexQuantization> GenomeEditCommandStep<FIQ> for GenomeEditCommandStepBrainRegionAdd {}

impl<FIQ: FeagiIndexQuantization> From<GenomeEditCommandStepBrainRegionAdd> for GenomeEditCommandStepEnum<FIQ> {
    fn from(value: GenomeEditCommandStepBrainRegionAdd) -> Self {
        GenomeEditCommandStepEnum::<FIQ>::BrainRegionAdd(value)
    }
}