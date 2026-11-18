use crate::basis::brain_region::{BrainRegionID, BrainRegionIdAndAncestryConstructor};
use crate::genome::editing::genome_edit_requests::GenomeEditSequenceIntensity;
use crate::genome::editing::steps::{GenomeEditStep, GenomeEditStepEnumAdditive, GenomeEditStepEnumAny};
use crate::genome::genome_object::{GenomeObjectMetadata, GenomeObjectPositioning};
use feagi_connectome::connectome_basis::quantization_levels::FeagiIndexQuantization;
use serde::{Deserialize, Serialize};


/// Adds a brain region with the given details. if optional_parent_region is None, the region is
/// added to the root brain region
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenomeEditStepBrainRegionAdd {
    pub metadata: GenomeObjectMetadata,
    pub positioning: GenomeObjectPositioning,
    pub this_ids: BrainRegionIdAndAncestryConstructor,
    pub optional_parent_region: Option<BrainRegionID>,
}

impl<FIQ: FeagiIndexQuantization> GenomeEditStep<FIQ> for GenomeEditStepBrainRegionAdd {
    const INTENSITY: GenomeEditSequenceIntensity = GenomeEditSequenceIntensity::GenomeOnly;
}

impl<FIQ: FeagiIndexQuantization> From<GenomeEditStepBrainRegionAdd> for GenomeEditStepEnumAdditive<FIQ> {
    fn from(value: GenomeEditStepBrainRegionAdd) -> Self {
        GenomeEditStepEnumAdditive::AddBrainRegion(value)
    }
}

impl<FIQ: FeagiIndexQuantization> From<GenomeEditStepBrainRegionAdd> for GenomeEditStepEnumAny<FIQ> {
    fn from(value: GenomeEditStepBrainRegionAdd) -> Self {
        GenomeEditStepEnumAny::AddBrainRegion(value)
    }
}