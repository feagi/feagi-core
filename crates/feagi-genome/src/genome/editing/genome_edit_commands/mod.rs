use feagi_connectome::connectome_basis::quantization_levels::FeagiIndexQuantization;
use feagi_connectome::connectome_burst::burst_basis::connectome_editing::{ConnectomeEditEnum, ConnectomeEditNothing};
use crate::basis::brain_region::BrainRegionID;
use crate::basis::cortical_area::cortical_identifiers::{CorticalIdentifier, CorticalIdentifierInnerEncoder};
use crate::basis::cortical_area::{CorticalType, CorticalTypedSpawningParameters};
use crate::basis::cortical_area::foundation::{CTFoundationSubtypedSpawningParameters, CorticalIdentifierFoundation};
use crate::basis::cortical_area::foundation::power::CTFoundationPower;
use crate::basis::cortical_area::processing::{CTProcessingSubtypedSpawningParameters, CorticalIdentifierProcessing};
use crate::basis::cortical_area::processing::interconnect::CTProcessingInterconnect;
use crate::genome::editing::genome_edit_commands::brain_region::GenomeEditCommandStepBrainRegionAdd;
use crate::genome::editing::genome_edit_commands::cortical_area::GenomeEditCommandStepCorticalAreaAdd;
use crate::genome::editing::steps::{GenomeEditStepEnum, GenomeEditStepEnumAny};
use crate::genome::genomes::brain_region::BrainRegionContainerUniversal;
use crate::genome::genomes::GenomePublicInterface;

mod brain_region;
mod cortical_area;

/// Used internally, handles validating genome edit requests, adding missing steps, and sending
/// edits to connectome
pub(crate) struct GenomeEditCommands<FIQ: FeagiIndexQuantization> {
    pub(crate) commands: Vec<GenomeEditCommandStepEnum<FIQ>>,
}

impl<FIQ: FeagiIndexQuantization> GenomeEditCommands<FIQ> {

    /// Iterates through requests, adding command(s) for each after validating that they are valid
    pub fn validate_requests_into_commands<'a, StepEnum>(
        requests: impl IntoIterator<Item = &'a StepEnum>,
        genome: &impl GenomePublicInterface<FIQ>
    ) -> Result<Self, GenomeEditRequestValidationError<FIQ>>
    where
        StepEnum: GenomeEditStepEnum<FIQ> + 'a,
    {

        //region Sub Functions

        /// Given None is get the root, and a value is something we need to verify exists and get, get the parent brain region id
        fn get_parent_region<FIQ: FeagiIndexQuantization>(id: Option<BrainRegionID>, genome: &impl GenomePublicInterface<FIQ>) -> Result<BrainRegionID, GenomeEditRequestValidationError<FIQ>> {
            match id {
                Some(id) => match genome.get_brain_region_context(&id) {
                    // The parent must already exist in the genome
                    Ok(_) => Ok(id),
                    Err(_) => Err(GenomeEditRequestValidationError::InvalidParentBrainRegion),
                },
                None => {
                    // Get the root
                    Ok(genome.get_root_region_container().get_root_id().clone())
                }
            }
        }



        //endregion

        let mut commands: Vec<GenomeEditCommandStepEnum<FIQ>> = vec![];
        // NOTE: Its possible that the number of commands exceeds the number of requests, since
        // one request may need multiple commands
        // (deleting a cortical area may need mappings cleared first)

        for request in requests.into_iter(){
            let request_step: GenomeEditStepEnumAny<FIQ> = request.clone().into();

            match request_step {
                GenomeEditStepEnumAny::AddBrainRegion(a) => {

                    // Confirm parent region exists and get it
                    let parent_region = get_parent_region(a.optional_parent_region, genome)?;

                    // TODO check for valid metadata

                    // TODo Check for valid positioning

                    let command = GenomeEditCommandStepEnum::AddBrainRegion(
                        GenomeEditCommandStepBrainRegionAdd {
                            metadata: a.metadata,
                            positioning: a.positioning,
                            this_ids: a.this_ids,
                            parent_region,
                        }
                    );
                    commands.push(command);
                }
                GenomeEditStepEnumAny::AddCorticalArea(a) => {

                    // Confirm parent region exists and get it
                    let parent_region = get_parent_region(a.parent_region , genome)?;

                    // Check that the parent region is allowed for the cortical type
                    match genome.get_brain_region_context(&parent_region).unwrap().is_root_region() {
                        true => {
                            // parent region is root
                            if !a.neurogenesis.get_allowed_regions().is_root_allowed() {
                                return Err(GenomeEditRequestValidationError::CorticalAreaCannotBeInRootRegion)
                            }
                        }
                        false => {
                            // parent region is inner
                            if !a.neurogenesis.get_allowed_regions().is_inner_allowed() {
                                return Err(GenomeEditRequestValidationError::CorticalAreaCannotBeInInnerRegion)
                            }
                        }
                    }

                    // TODO find a better / cleaner way to do this cortical ID stuff
                    // TODO check genome collision!
                    // generate cortical id, validate cortical ID
                    let cortical_identifier = match &a.neurogenesis {
                        CorticalTypedSpawningParameters::Foundation(st) => {
                            match st {
                                CTFoundationSubtypedSpawningParameters::Power(ct) => {
                                    let params = ct.cortical_id_generation_params;
                                    let cortical_id_inner = <CTFoundationPower as CorticalType>::InnerCorticalIDGenerator::encode_context(());
                                    CorticalIdentifier::Foundation(CorticalIdentifierFoundation::Power(cortical_id_inner))
                                }
                            }
                        }
                        CorticalTypedSpawningParameters::Processing(st) => {
                            match st {
                                CTProcessingSubtypedSpawningParameters::Interconnect(ct) => {
                                    let params = ct.cortical_id_generation_params;
                                    let cortical_id_inner = <CTProcessingInterconnect as CorticalType>::InnerCorticalIDGenerator::encode_context(());
                                    CorticalIdentifier::Processing(CorticalIdentifierProcessing::Interconnect(cortical_id_inner))
                                }
                            }
                        }
                    };

                    // TODO check if quantization values are possible if designer genome with npu

                    // TODO check for valid metadata

                    // TODO Check for valid positioning

                    let command = GenomeEditCommandStepEnum::AddCorticalArea(
                        GenomeEditCommandStepCorticalAreaAdd {
                            metadata: a.metadata,
                            positioning: a.positioning,
                            cortical_identifier,
                            neurogenesis: a.neurogenesis,
                            parent_region
                        }
                    );

                    commands.push(command);

                }
            }



        };


        Ok(Self {
            commands
        })

    }



}


pub(crate) trait GenomeEditCommandStep<FIQ: FeagiIndexQuantization> {
    
    fn generate_connectome_edit_step(&self) -> ConnectomeEditEnum<FIQ> {
        ConnectomeEditEnum::Nothing(ConnectomeEditNothing::new()) // by default, do nothing
    }
}

pub(crate) enum GenomeEditCommandStepEnum<FIQ: FeagiIndexQuantization> {
    AddBrainRegion(GenomeEditCommandStepBrainRegionAdd),
    AddCorticalArea(GenomeEditCommandStepCorticalAreaAdd<FIQ>)
}


#[derive(Debug, thiserror::Error)]
pub enum GenomeEditRequestValidationError<FIQ: FeagiIndexQuantization> {
    #[error("Unable to parent to BrainRe")]
    InvalidParentBrainRegion,
    #[error("Adding Cortical Area Type cannot be added to the root region!")]
    CorticalAreaCannotBeInRootRegion,
    #[error("Adding Cortical Area Type cannot be added to a non-root region!")]
    CorticalAreaCannotBeInInnerRegion,

    #[error("Disallowed Dimensions!")]
    CorticalAreaDimensionsInvalid(FIQ) // TODO make an actual thing
}