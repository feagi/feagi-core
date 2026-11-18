use serde::{Deserialize, Serialize};
use feagi_connectome::connectome_basis::quantization_levels::FeagiIndexQuantization;
use feagi_connectome::connectome_burst::burst_basis::connectome_editing::ConnectomeEditEnum;
use feagi_connectome::connectome_burst::burst_basis::connectome_reshaping::{ConnectomeReshapeStepCorticalAreaAdd, ConnectomeReshapeStepEnum};
use feagi_connectome::connectome_models::neuron_models::models::exclusive_power::quantizations::NMExclusivePowerQuanted;
use feagi_connectome::connectome_models::neuron_models::models::feagi_advanced::quantizations::NMFeagiAdvancedQuanted;
use feagi_connectome::connectome_models::neuron_models::models::feagi_simple::quantizations::NMFeagiSimpleQuanted;
use feagi_connectome::connectome_models::neuron_models::models::NMNeurogenesisStorageOrGenerator;
use crate::basis::brain_region::BrainRegionID;
use crate::basis::cortical_area::cortical_identifiers::CorticalIdentifier;
use crate::basis::cortical_area::{CorticalTypeAreaSpawningParameters, CorticalTypedSpawningParameters};
use crate::basis::cortical_area::foundation::CTFoundationSubtypedSpawningParameters;
use crate::basis::cortical_area::foundation::power::{CTFoundationPower, CTFoundationPowerNMModeledStorageOrGenerator};
use crate::basis::cortical_area::processing::CTProcessingSubtypedSpawningParameters;
use crate::basis::cortical_area::processing::interconnect::{CTProcessingInterconnect, CTProcessingInterconnectNMModeledStorageOrGenerator};
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
    fn generate_connectome_edit_step(&self) -> ConnectomeEditEnum<FIQ> {

        let temp = self.neurogenesis.clone();
        let neurogenesis = match temp {
            CorticalTypedSpawningParameters::Foundation(a) => {
                match a {
                    CTFoundationSubtypedSpawningParameters::Power(b) => {
                        match b {
                            CorticalTypeAreaSpawningParameters { cortical_id_generation_params, neurogenesis } => {
                                match neurogenesis {
                                    CTFoundationPowerNMModeledStorageOrGenerator::ExclusivePower(c) => {
                                        match c {
                                            NMExclusivePowerQuanted::Float16(d) => {}
                                            NMExclusivePowerQuanted::BFloat16(d) => {}
                                            NMExclusivePowerQuanted::Float32(d) => {}
                                            NMExclusivePowerQuanted::Float64(d) => {}
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            CorticalTypedSpawningParameters::Processing(a) => {
                match a {
                    CTProcessingSubtypedSpawningParameters::Interconnect(b) => {
                        match b {
                            CorticalTypeAreaSpawningParameters { cortical_id_generation_params, neurogenesis } => {
                                match neurogenesis {
                                    CTProcessingInterconnectNMModeledStorageOrGenerator::FeagiAdvanced(c) => {
                                        match c {
                                            NMFeagiAdvancedQuanted::Standard32Bit(d) => {}
                                        }
                                    }
                                    CTProcessingInterconnectNMModeledStorageOrGenerator::FeagiSimple(c) => {
                                        match c {
                                            NMFeagiSimpleQuanted::Standard32Bit(d) => {}
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        ConnectomeEditEnum::EditConnectomeStep(
            ConnectomeReshapeStepEnum::CorticalAreaAdd(
                ConnectomeReshapeStepCorticalAreaAdd {
                    cortical_area_neurogenesis: neurogenesis }
            )
        )
    }
}
