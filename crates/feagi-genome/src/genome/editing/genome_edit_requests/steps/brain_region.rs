use crate::basis::brain_region::{BrainRegionAncestryID, BrainRegionID, BrainRegionIDType, FauxBrainRegionID, MaybeBrainRegionID};
use crate::genome::editing::genome_edit_commands::{GenomeEditCommandStepEnum};
use crate::genome::editing::genome_edit_requests::steps::{
    GenomeEditStep, GenomeEditStepEnumAdditive, GenomeEditStepEnumAny, GenomeEditStepInternal,
};
use crate::genome::editing::genome_edit_requests::GenomeEditSequenceIntensity;
use crate::genome::genome_object::{GenomeObjectMetadata, GenomeObjectPositioning};
use crate::genome::genomes::{GenomePublicInterface};
use feagi_connectome::connectome_basis::quantization_levels::FeagiIndexQuantization;
use feagi_connectome::connectome_burst::burst_basis::connectome_editing::{ConnectomeEditEnum, ConnectomeEditNothing};
use serde::{Deserialize, Serialize};
use crate::genome::editing::genome_edit_commands::brain_region::GenomeEditCommandStepBrainRegionAdd;
use crate::genome::genomes::genome_editing::{GenomeEditCommandCollisionDetection, GenomeEditingError};


/// Adds a brain region with the given details. The faux brain region is used for mappings and may
/// not be translated into the real one. The optional ancestry id if given will be used, regardless
/// of repeats, and if none will be generated. The parent region is controlled via a generic, so
/// what it can be added to is controlled via context
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct GenomeEditStepBrainRegionAdd<Parent: BrainRegionIDType> {
    pub metadata: GenomeObjectMetadata,
    pub positioning: GenomeObjectPositioning,
    pub faux_brain_region_id: FauxBrainRegionID,
    pub ancestry_id: Option<BrainRegionAncestryID>,
    pub parent_region: Parent,
}

impl<FIQ: FeagiIndexQuantization, Parent: BrainRegionIDType> GenomeEditStep<FIQ> for GenomeEditStepBrainRegionAdd<Parent> {
    const INTENSITY: GenomeEditSequenceIntensity = GenomeEditSequenceIntensity::GenomeOnly;
}

/// In the case of amalgamation or reload
impl<FIQ: FeagiIndexQuantization> GenomeEditStepInternal<FIQ> for GenomeEditStepBrainRegionAdd<FauxBrainRegionID> {
    fn verify_with_genome_and_collider_and_push_genome_commands(
        self,
        genome: &impl GenomePublicInterface<FIQ>,
        collision: &mut GenomeEditCommandCollisionDetection,
        genome_commands: &mut Vec<GenomeEditCommandStepEnum<FIQ>>,
    ) -> Result<(), GenomeEditingError> {
        
        let new_self_id = collision.convert_id_to_add_brain_region(genome, self.faux_brain_region_id)?;
        let parent_id = collision.get_parent_brain_region_from_faux(self.parent_region)?;
        
        // TODO collision logic? Metadata checks?

        let command = GenomeEditCommandStepBrainRegionAdd {
            metadata: self.metadata,
            positioning: self.positioning,
            this_id: new_self_id,
            parent_region_id: parent_id,
        };
        
        genome_commands.push(command.into());
        Ok(())
    }

    fn verify_with_genome_and_collider_and_push_genome_and_connectome_commands(
        self,
        genome: &impl GenomePublicInterface<FIQ>,
        collision: &mut GenomeEditCommandCollisionDetection,
        genome_commands: &mut Vec<GenomeEditCommandStepEnum<FIQ>>,
        connectome_commands: &mut Vec<ConnectomeEditEnum<FIQ>>,
    ) -> Result<(), GenomeEditingError> {
        self.verify_with_genome_and_collider_and_push_genome_commands(
            genome,
            collision,
            genome_commands,
        )?;
        connectome_commands.push(ConnectomeEditEnum::Nothing(ConnectomeEditNothing::new()));
        Ok(())
    }
}


