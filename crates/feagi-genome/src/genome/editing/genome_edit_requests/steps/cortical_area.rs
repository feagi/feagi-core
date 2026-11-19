use crate::basis::brain_region::{BrainRegionID, BrainRegionIDType, FauxBrainRegionID, MaybeBrainRegionID};
use crate::basis::cortical_area::CorticalTypedSpawningRequest;
use crate::genome::editing::genome_edit_commands::GenomeEditCommandStepEnum;
use crate::genome::editing::genome_edit_requests::steps::{
    GenomeEditStep, GenomeEditStepEnumAdditive, GenomeEditStepEnumAny, GenomeEditStepInternal,
};
use crate::genome::editing::genome_edit_requests::GenomeEditSequenceIntensity;
use crate::genome::genome_object::{GenomeObjectMetadata, GenomeObjectPositioning};
use crate::genome::genomes::genome_editing::{GenomeEditCommandCollisionDetection, GenomeEditingError};
use crate::genome::genomes::GenomePublicInterface;
use feagi_connectome::connectome_basis::quantization_levels::FeagiIndexQuantization;
use feagi_connectome::connectome_burst::burst_basis::connectome_editing::ConnectomeEditEnum;
use serde::{Deserialize, Serialize};

/// The enum edit step for cortical areas for the load genome sequence type. Can only add cortical areas
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct GenomeEditStepCorticalAreaAdd<FIQ: FeagiIndexQuantization, Parent: BrainRegionIDType> {
    pub metadata: GenomeObjectMetadata,
    pub positioning: GenomeObjectPositioning,
    pub parent_region: Parent,
    pub spawner: CorticalTypedSpawningRequest<FIQ>,
}

impl<FIQ: FeagiIndexQuantization, Parent: BrainRegionIDType> GenomeEditStep<FIQ> for GenomeEditStepCorticalAreaAdd<FIQ, Parent> {
    const INTENSITY: GenomeEditSequenceIntensity = GenomeEditSequenceIntensity::ConnectomeAllocating;
}

/// For Amalgamation / Additive
impl<FIQ: FeagiIndexQuantization> GenomeEditStepInternal<FIQ> for GenomeEditStepCorticalAreaAdd<FIQ, FauxBrainRegionID> {
    
    fn verify_with_genome_and_collider_and_push_genome_commands(
        self,
        genome: &impl GenomePublicInterface<FIQ>,
        collision: &mut GenomeEditCommandCollisionDetection,
        genome_commands: &mut Vec<GenomeEditCommandStepEnum<FIQ>>,
    ) -> Result<(), GenomeEditingError> {
        
        // check / get id
        collision.convert_id_to_add_area(self.)
        
    }

    fn verify_with_genome_and_collider_and_push_genome_and_connectome_commands(
        self,
        genome: &impl GenomePublicInterface<FIQ>,
        collision: &mut GenomeEditCommandCollisionDetection,
        genome_commands: &mut Vec<GenomeEditCommandStepEnum<FIQ>>,
        connectome_commands: &mut Vec<ConnectomeEditEnum<FIQ>>,
    ) -> Result<(), GenomeEditingError> {
        todo!()
    }
}

impl<FIQ: FeagiIndexQuantization> From<GenomeEditStepCorticalAreaAdd<FIQ, FauxBrainRegionID>> for GenomeEditStepEnumAdditive<FIQ> {
    fn from(value: GenomeEditStepCorticalAreaAdd<FIQ, FauxBrainRegionID>) -> Self {
        GenomeEditStepEnumAdditive::AddCorticalArea(value)
    }
}






impl<FIQ: FeagiIndexQuantization> From<GenomeEditStepCorticalAreaAdd<FIQ, MaybeBrainRegionID>> for GenomeEditStepEnumAny<FIQ> {
    fn from(value: GenomeEditStepCorticalAreaAdd<FIQ, MaybeBrainRegionID>) -> Self {
        GenomeEditStepEnumAny::AddCorticalArea(value)
    }
}
