use serde::{Deserialize, Serialize};
use feagi_connectome::connectome_basis::quantization_levels::FeagiIndexQuantization;
use crate::basis::brain_region::BrainRegionID;
use crate::basis::cortical_area::cortical_identifiers::CorticalIdentifier;
use crate::basis::cortical_area::CorticalTypedSpawningParameters;
use crate::genome::editing::genome_edit_commands::GenomeEditCommandStep;
use crate::genome::genome_object::{GenomeObjectMetadata, GenomeObjectPositioning};

/// The enum edit step for cortical areas for the load genome sequence type. Can only add cortical areas
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct GenomeEditCommandStepCorticalAreaAdd<FIQ: FeagiIndexQuantization> {
    pub metadata: GenomeObjectMetadata,
    pub positioning: GenomeObjectPositioning,
    pub cortical_identifier: CorticalIdentifier,
    pub neurogenesis: CorticalTypedSpawningParameters<FIQ>, // TODO swap to one that holds the cortical id (not cortical id generator) (perhaps making a generic works for holding that context?)
    pub parent_region: BrainRegionID
}

impl<FIQ: FeagiIndexQuantization> GenomeEditCommandStep<FIQ> for GenomeEditCommandStepCorticalAreaAdd<FIQ> {
}
