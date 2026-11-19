pub mod brain_region;
pub mod cortical_area;


use crate::genome::editing::genome_edit_requests::steps::cortical_area::GenomeEditStepCorticalAreaAdd;
use crate::genome::editing::genome_edit_requests::GenomeEditSequenceIntensity;
use feagi_connectome::connectome_basis::quantization_levels::FeagiIndexQuantization;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use feagi_connectome::connectome_burst::burst_basis::connectome_editing::ConnectomeEditEnum;
use crate::genome::editing::genome_edit_commands::{GenomeEditCommandStepEnum};
use crate::genome::editing::genome_edit_requests::steps::brain_region::GenomeEditStepBrainRegionAdd;
use crate::genome::genomes::genome_editing::{GenomeEditCommandCollisionDetection, GenomeEditingError};
use crate::genome::genomes::GenomePublicInterface;
// TODO macros maybe? One for each enum and one for the types?

/// A single edit step stored under `GenomeEditRequests`.
pub trait GenomeEditStep<FIQ>: Clone + Debug + Serialize + DeserializeOwned
where
    FIQ: FeagiIndexQuantization,
{
    const INTENSITY: GenomeEditSequenceIntensity;
}


pub(crate) trait GenomeEditStepInternal<FIQ: FeagiIndexQuantization>: Clone + Debug + Serialize + DeserializeOwned {

    /// Verify with Genome and building collider that this step is valid and add commands
    fn verify_with_genome_and_collider_and_push_genome_commands(
        self,
        genome: &impl GenomePublicInterface<FIQ>,
        collision: &mut GenomeEditCommandCollisionDetection,
        genome_commands: &mut Vec<GenomeEditCommandStepEnum<FIQ>>
    ) -> Result<(), GenomeEditingError>;

    /// Verify with Genome and building collider that this step is valid and add commands for genome and connectome
    fn verify_with_genome_and_collider_and_push_genome_and_connectome_commands(
        self,
        genome: &impl GenomePublicInterface<FIQ>,
        collision: &mut GenomeEditCommandCollisionDetection,
        genome_commands: &mut Vec<GenomeEditCommandStepEnum<FIQ>>,
        connectome_commands: &mut Vec<ConnectomeEditEnum<FIQ>>
    ) -> Result<(), GenomeEditingError>;
}



pub trait GenomeEditStepEnum<FIQ: FeagiIndexQuantization>:
Clone + Debug + Serialize + DeserializeOwned + Into<GenomeEditStepEnumAny<FIQ>>
{
    fn get_intensity(&self) -> GenomeEditSequenceIntensity;
}


/// Allows Edit actions that only add to a genome, not anything that changes or removes anything
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum GenomeEditStepEnumAdditive<FIQ: FeagiIndexQuantization> {
    AddBrainRegion(GenomeEditStepBrainRegionAdd<Parent>),
    AddCorticalArea(GenomeEditStepCorticalAreaAdd<FIQ>)
}

impl<FIQ: FeagiIndexQuantization> GenomeEditStepEnum<FIQ> for GenomeEditStepEnumAdditive<FIQ> {
    fn get_intensity(&self) -> GenomeEditSequenceIntensity {
        match self {
            GenomeEditStepEnumAdditive::AddBrainRegion(_) => <GenomeEditStepBrainRegionAdd as GenomeEditStep<FIQ>>::INTENSITY,
            GenomeEditStepEnumAdditive::AddCorticalArea(_) => <GenomeEditStepCorticalAreaAdd<FIQ> as GenomeEditStep<FIQ>>::INTENSITY,
        }
    }
}



/// Allows effectively any edit action, such as adding, removing, or changing properties
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum GenomeEditStepEnumAny<FIQ: FeagiIndexQuantization> {
    AddBrainRegion(GenomeEditStepBrainRegionAdd),
    AddCorticalArea(GenomeEditStepCorticalAreaAdd<FIQ>)
}

impl<FIQ: FeagiIndexQuantization> GenomeEditStepEnum<FIQ> for GenomeEditStepEnumAny<FIQ> {
    fn get_intensity(&self) -> GenomeEditSequenceIntensity {
        match self {
            GenomeEditStepEnumAny::AddBrainRegion(_) => <GenomeEditStepBrainRegionAdd as GenomeEditStep<FIQ>>::INTENSITY,
            GenomeEditStepEnumAny::AddCorticalArea(_) => <GenomeEditStepCorticalAreaAdd<FIQ> as GenomeEditStep<FIQ>>::INTENSITY,
        }
    }
}

impl<FIQ: FeagiIndexQuantization> From<GenomeEditStepEnumAdditive<FIQ>> for GenomeEditStepEnumAny<FIQ> {
    fn from(value: GenomeEditStepEnumAdditive<FIQ>) -> Self {
        match value {
            GenomeEditStepEnumAdditive::AddBrainRegion(a) => {
                GenomeEditStepEnumAny::AddBrainRegion(a)
            }
            GenomeEditStepEnumAdditive::AddCorticalArea(a) => {
                GenomeEditStepEnumAny::AddCorticalArea(a)
            }
        }
    }
}