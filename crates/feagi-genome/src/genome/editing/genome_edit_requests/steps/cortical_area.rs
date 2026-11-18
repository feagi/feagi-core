use crate::basis::brain_region::BrainRegionID;
use crate::basis::cortical_area::cortical_identifiers::FauxCorticalIdentifier;
use crate::basis::cortical_area::CorticalTypedSpawningParameters;
use crate::genome::editing::genome_edit_requests::steps::{GenomeEditStep, GenomeEditStepEnumAdditive, GenomeEditStepEnumAny};
use crate::genome::editing::genome_edit_requests::GenomeEditSequenceIntensity;
use crate::genome::genome_object::{GenomeObjectMetadata, GenomeObjectPositioning};
use feagi_connectome::connectome_basis::quantization_levels::FeagiIndexQuantization;
use serde::{Deserialize, Serialize};

/// The enum edit step for cortical areas for the load genome sequence type. Can only add cortical areas
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct GenomeEditStepCorticalAreaAdd<FIQ: FeagiIndexQuantization> {
    metadata: GenomeObjectMetadata,
    positioning: GenomeObjectPositioning,
    faux_cortical_identifier: FauxCorticalIdentifier,
    neurogenesis: CorticalTypedSpawningParameters<FIQ>, // TODO swap
    parent_region: Option<BrainRegionID> // TODO verify should ensure that type matches target
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