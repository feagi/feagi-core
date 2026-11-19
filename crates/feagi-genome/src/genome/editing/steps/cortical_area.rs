use crate::basis::brain_region::BrainRegionID;
use crate::basis::cortical_area::cortical_identifiers::FauxCorticalIdentifier;
use crate::genome::editing::genome_edit_requests::GenomeEditSequenceIntensity;
use crate::genome::editing::steps::{GenomeEditStep, GenomeEditStepEnumAdditive, GenomeEditStepEnumAny};
use crate::genome::genome_object::{GenomeObjectMetadata, GenomeObjectPositioning};
use feagi_connectome::connectome_basis::quantization_levels::FeagiIndexQuantization;
use serde::{Deserialize, Serialize};
use crate::basis::cortical_area::CorticalTypedSpawningRequest;

/// The enum edit step for cortical areas for the load genome sequence type. Can only add cortical areas
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct GenomeEditStepCorticalAreaAdd<FIQ: FeagiIndexQuantization> {
    pub metadata: GenomeObjectMetadata,
    pub positioning: GenomeObjectPositioning,
    pub parent_region: Option<BrainRegionID>,
    pub spawner: CorticalTypedSpawningRequest<FIQ>,
    
}

impl<FIQ: FeagiIndexQuantization> GenomeEditStep<FIQ> for GenomeEditStepCorticalAreaAdd<FIQ> {
    const INTENSITY: GenomeEditSequenceIntensity = GenomeEditSequenceIntensity::ConnectomeAllocating;
}

impl<FIQ: FeagiIndexQuantization> From<GenomeEditStepCorticalAreaAdd<FIQ>> for GenomeEditStepEnumAdditive<FIQ> {
    fn from(value: GenomeEditStepCorticalAreaAdd<FIQ>) -> Self {
        GenomeEditStepEnumAdditive::AddCorticalArea(value)
    }
}

impl<FIQ: FeagiIndexQuantization> From<GenomeEditStepCorticalAreaAdd<FIQ>> for GenomeEditStepEnumAny<FIQ> {
    fn from(value: GenomeEditStepCorticalAreaAdd<FIQ>) -> Self {
        GenomeEditStepEnumAny::AddCorticalArea(value)
    }
}