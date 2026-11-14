use serde::{Deserialize, Serialize};
use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::{NeuronLayoutRestrictor, NeuronLayoutTag};
use crate::basis::cortical_area::{CorticalType};
use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome::feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
use crate::basis::cortical_area::cortical_identifiers::{CorticalAreaInnerID, CorticalAreaInnerIDRandomBytes, CorticalIdentifierInner, CorticalIdentifierParseError};
use crate::basis::cortical_area::foundation::power::CTFoundationPower;
use crate::basis::cortical_area::processing::interconnect::{CTProcessingInterconnectNMModeledGeneratorPrecursor, CTProcessingInterconnectNMModeledKind, CTProcessingInterconnectNMModeledStorage, CTProcessingInterconnectNMModeledStorageOrGeneratorPrecursor};


//region Cortical Subtype

/// This cortical type is for
pub trait CorticalTypeProcessing<NM>: CorticalType<NM, InnerCorticalIDGenerator=[u8; 6]> // uses 6 bytes as cortical ID inner
where
    NM: NeuronModel<NeuronLayoutTag=Self::NeuronLayout>
{}

/// The most common cortical type for general processing between inputs and outputs
pub mod interconnect {

    //region Supported Neuron Models for Interconnect

    use serde::{Deserialize, Serialize};
    use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::{NeuronLayoutOf, NeuronLayoutRestrictor};
    use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
    use feagi_connectome::feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
    use feagi_connectome::neuron_models::models::feagi_advanced::quantizations::NMFeagiAdvancedQuantedKind;
    use feagi_connectome::neuron_models::models::feagi_simple::quantizations::NMFeagiSimpleQuantedKind;
    use crate::basis::cortical_area::processing::interconnect::feagi_advanced::{CTProcessingInterconnectNMFeagiAdvancedQuantedGeneratorPrecursor, CTProcessingInterconnectNMFeagiAdvancedQuantedKind, CTProcessingInterconnectNMFeagiAdvancedQuantedStorage, CTProcessingInterconnectNMFeagiAdvancedQuantedStorageOrGeneratorPrecursor};
    use crate::basis::cortical_area::processing::interconnect::feagi_simple::{CTProcessingInterconnectNMFeagiSimpleQuantedGeneratorPrecursor, CTProcessingInterconnectNMFeagiSimpleQuantedKind, CTProcessingInterconnectNMFeagiSimpleQuantedStorage, CTProcessingInterconnectNMFeagiSimpleQuantedStorageOrGeneratorPrecursor};

    /// The NMs / Quants allowed for this Cortical Type
    pub trait CTProcessingInterconnectNM: NeuronModel
    {
        /// Spawns the generator precursor, effectively the default parameters for creating a cortical area
        fn spawn_neurogenesis_generator_preview<
            FIQ: FeagiIndexQuantization,
            Res: NeuronLayoutRestrictor<FIQ, NeuronLayout=NeuronLayoutOf<FIQ, Self>>
        >() -> Self::NeurogenesisGeneratorPrecursor<FIQ>;
    }

    /// Default dimensions to use for this cortical type
    const DEFAULT_DIMENSIONS: [usize; 4] = [10, 10, 10, 1]; // TODO direct const of DimensionalCorticalAreaDimensions


    pub mod feagi_advanced {
        use serde::{Deserialize, Serialize};
        use feagi_basis::neurons::{CorticalAreaNeuronPotential, DimensionalCorticalAreaDimensions};
        use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::model_data::NeuronModelUniversalCorticalProperties;
        use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::from_storage::NeurogenesisStorage;
        use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::{NeuronLayoutOf, NeuronLayoutRestrictor};
        use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
        use feagi_connectome::feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
        use feagi_connectome::neuron_models::models::feagi_advanced::{NMFeagiAdvancedDataCorticalProperties, NMFeagiAdvancedDataCorticalVariance, NMFeagiAdvancedDataNeuronInternal};
        use feagi_connectome::neuron_models::models::feagi_advanced::quantizations::{NMFeagiAdvancedNeurogenesisStorageOrGeneratorQuanted, NMFeagiAdvancedQuantStandard32Bit, NMFeagiAdvancedQuantedKind, NMFeagiAdvancedNeurogenesisGeneratorPrecursorQuanted};
        use crate::basis::cortical_area::processing::interconnect::{CTProcessingInterconnectNM, DEFAULT_DIMENSIONS};

        impl CTProcessingInterconnectNM for NMFeagiAdvancedQuantStandard32Bit {
            fn spawn_neurogenesis_generator_preview<
                FIQ: FeagiIndexQuantization,
                Res: NeuronLayoutRestrictor<FIQ, NeuronLayout=NeuronLayoutOf<FIQ, Self>>>
            () -> Self::NeurogenesisGeneratorPrecursor<FIQ> {
                use feagi_connectome::neuron_models::models::feagi_advanced::wrapped::Period;

                Self::NeurogenesisGeneratorPrecursor {
                    universal_properties: NeuronModelUniversalCorticalProperties {
                        cortical_level_psp: CorticalAreaNeuronPotential::QUANT_ONE,
                        is_psp_uniform: false,
                        is_psp_using_neuron_potential: false,
                    },
                    cortical_context: DimensionalCorticalAreaDimensions::new_dimensions_from_usize(DEFAULT_DIMENSIONS).unwrap(),
                    cortical_properties: NMFeagiAdvancedDataCorticalProperties {
                        refractory_period: Period::QUANT_ONE,
                        firing_threshold: CorticalAreaNeuronPotential::QUANT_ONE,
                        leak_coefficient: CorticalAreaNeuronPotential::QUANT_ZERO,
                        leak_variability: CorticalAreaNeuronPotential::QUANT_ZERO,
                        postsynaptic_current: CorticalAreaNeuronPotential::QUANT_ZERO,
                        postsynaptic_current_max: CorticalAreaNeuronPotential::QUANT_ZERO,
                        degeneration: CorticalAreaNeuronPotential::QUANT_ZERO,
                    },
                    cortical_internal: Default::default(),
                    cortical_variance: NMFeagiAdvancedDataCorticalVariance {
                        firing_threshold_increment_x: CorticalAreaNeuronPotential::QUANT_ZERO,
                        firing_threshold_increment_y: CorticalAreaNeuronPotential::QUANT_ZERO,
                        firing_threshold_increment_z: CorticalAreaNeuronPotential::QUANT_ZERO,
                    },
                    neuron_internal: NMFeagiAdvancedDataNeuronInternal {
                        refractory_countdown: Period::QUANT_ONE,
                    }
                }
            }
        }
        
        #[derive(Copy, Clone, Debug)]
        pub enum CTProcessingInterconnectNMFeagiAdvancedQuantedKind {
            Standard32Bit(NMFeagiAdvancedQuantedKind)
        }

        #[derive(Debug, Clone, Serialize, Deserialize)]
        #[serde(bound = "")]
        pub enum CTProcessingInterconnectNMFeagiAdvancedQuantedGeneratorPrecursor<FIQ: FeagiIndexQuantization> {
            Standard32Bit(<NMFeagiAdvancedQuantStandard32Bit as NeuronModel>::NeurogenesisGeneratorPrecursor<FIQ>),
        }

        #[derive(Debug, Clone, Serialize, Deserialize)]
        #[serde(bound = "")]
        pub enum CTProcessingInterconnectNMFeagiAdvancedQuantedStorage<FIQ: FeagiIndexQuantization> {
            Standard32Bit(NeurogenesisStorage<FIQ, NMFeagiAdvancedQuantStandard32Bit>),
        }

        #[derive(Debug, Clone, Serialize, Deserialize)]
        #[serde(bound = "")]
        pub enum CTProcessingInterconnectNMFeagiAdvancedQuantedStorageOrGeneratorPrecursor<FIQ: FeagiIndexQuantization> {
            Standard32Bit(NMFeagiAdvancedNeurogenesisGeneratorPrecursorQuanted<FIQ>),
        }
        

    }

    pub mod feagi_simple {
        use serde::{Deserialize, Serialize};
        use feagi_basis::neurons::{CorticalAreaNeuronPotential, DimensionalCorticalAreaDimensions};
        use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::model_data::NeuronModelUniversalCorticalProperties;
        use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::from_storage::NeurogenesisStorage;
        use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::{NeuronLayoutOf, NeuronLayoutRestrictor};
        use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
        use feagi_connectome::feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
        use feagi_connectome::neuron_models::models::feagi_simple::quantizations::{NMFeagiSimpleQuantedNeurogenesisGeneratorPrecursor, NMFeagiSimpleQuantStandard32Bit, NMFeagiSimpleQuantedKind};
        use crate::basis::cortical_area::processing::interconnect::{CTProcessingInterconnectNM, DEFAULT_DIMENSIONS};

        impl CTProcessingInterconnectNM for NMFeagiSimpleQuantStandard32Bit {
            fn spawn_neurogenesis_generator_preview<
                FIQ: FeagiIndexQuantization,
                Res: NeuronLayoutRestrictor<FIQ, NeuronLayout=NeuronLayoutOf<FIQ, Self>>>
            () -> Self::NeurogenesisGeneratorPrecursor<FIQ> {
                use feagi_connectome::neuron_models::models::feagi_simple::wrapped::*;
                use feagi_connectome::neuron_models::models::feagi_simple::*;

                Self::NeurogenesisGeneratorPrecursor {
                    universal_properties: NeuronModelUniversalCorticalProperties {
                        cortical_level_psp: CorticalAreaNeuronPotential::QUANT_ONE,
                        is_psp_uniform: false,
                        is_psp_using_neuron_potential: false,
                    },
                    cortical_context: DimensionalCorticalAreaDimensions::new_dimensions_from_usize(DEFAULT_DIMENSIONS).unwrap(),
                    cortical_properties: NMFeagiSimpleDataCorticalProperties {
                        refractory_period: Period::QUANT_ONE,
                        firing_threshold: CorticalAreaNeuronPotential::QUANT_ONE,
                        postsynaptic_current: CorticalAreaNeuronPotential::QUANT_ZERO,
                        postsynaptic_current_max: CorticalAreaNeuronPotential::QUANT_ZERO,
                    },
                    cortical_internal: Default::default(),
                    cortical_variance: NMFeagiSimpleDataCorticalVariance::default(),
                    neuron_internal: NMFeagiSimpleDataNeuronInternal {
                        refractory_countdown: Period::QUANT_ONE,
                    }
                }
            }
        }

        #[derive(Copy, Clone, Debug)]
        pub enum CTProcessingInterconnectNMFeagiSimpleQuantedKind {
            Standard32Bit(NMFeagiSimpleQuantedKind)
        }

        #[derive(Debug, Clone, Serialize, Deserialize)]
        #[serde(bound = "")]
        pub enum CTProcessingInterconnectNMFeagiSimpleQuantedGeneratorPrecursor<FIQ: FeagiIndexQuantization> {
            Standard32Bit(<NMFeagiSimpleQuantStandard32Bit as NeuronModel>::NeurogenesisGeneratorPrecursor<FIQ>),
        }

        #[derive(Debug, Clone, Serialize, Deserialize)]
        #[serde(bound = "")]
        pub enum CTProcessingInterconnectNMFeagiSimpleQuantedStorage<FIQ: FeagiIndexQuantization> {
            Standard32Bit(NeurogenesisStorage<FIQ, NMFeagiSimpleQuantStandard32Bit>),
        }

        #[derive(Debug, Clone, Serialize, Deserialize)]
        #[serde(bound = "")]
        pub enum CTProcessingInterconnectNMFeagiSimpleQuantedStorageOrGeneratorPrecursor<FIQ: FeagiIndexQuantization> {
            Standard32Bit(NMFeagiSimpleQuantedNeurogenesisGeneratorPrecursor<FIQ>),
        }
        
        
    }

    #[derive(Copy, Clone, Debug)]
    pub enum CTProcessingInterconnectNMModeledKind {
        FeagiAdvanced(CTProcessingInterconnectNMFeagiAdvancedQuantedKind),
        FeagiSimple(CTProcessingInterconnectNMFeagiSimpleQuantedKind),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(bound = "")]
    pub enum CTProcessingInterconnectNMModeledGeneratorPrecursor<FIQ: FeagiIndexQuantization> {
        FeagiAdvanced(CTProcessingInterconnectNMFeagiAdvancedQuantedGeneratorPrecursor<FIQ>),
        FeagiSimple(CTProcessingInterconnectNMFeagiSimpleQuantedGeneratorPrecursor<FIQ>),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(bound = "")]
    pub enum CTProcessingInterconnectNMModeledStorage<FIQ: FeagiIndexQuantization> {
        FeagiAdvanced(CTProcessingInterconnectNMFeagiAdvancedQuantedStorage<FIQ>),
        FeagiSimple(CTProcessingInterconnectNMFeagiSimpleQuantedStorage<FIQ>),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(bound = "")]
    pub enum CTProcessingInterconnectNMModeledStorageOrGeneratorPrecursor<FIQ: FeagiIndexQuantization> {
        FeagiAdvanced(CTProcessingInterconnectNMFeagiAdvancedQuantedStorageOrGeneratorPrecursor<FIQ>),
        FeagiSimple(CTProcessingInterconnectNMFeagiSimpleQuantedStorageOrGeneratorPrecursor<FIQ>),
    }
    
}

#[derive(Copy, Clone, Debug)]
pub enum CTProcessingSubtypedKind {
    Interconnect(CTProcessingInterconnectNMModeledKind),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum CTProcessingSubtypedGeneratorPrecursor<FIQ: FeagiIndexQuantization> {
    Interconnect(CTProcessingInterconnectNMModeledGeneratorPrecursor<FIQ>)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum CTProcessingSubtypedStorage<FIQ: FeagiIndexQuantization> {
    Interconnect(CTProcessingInterconnectNMModeledStorage<FIQ>)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum CTProcessingSubtypedStorageOrGeneratorPrecursor<FIQ: FeagiIndexQuantization> {
    Interconnect(CTProcessingInterconnectNMModeledStorageOrGeneratorPrecursor<FIQ>)
}

//endregion


/*
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum CTGeneratorProcessingInterconnectModeled<FIQ: FeagiIndexQuantization> {
    FeagiSimple(NMFeagiSimpleNeurogenesisGeneratorQuanted<FIQ>),
    FeagiAdvanced(NMFeagiAdvancedNeurogenesisGeneratorQuanted<FIQ>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum CTStorageProcessingInterconnectModeled<FIQ: FeagiIndexQuantization> {
    FeagiSimple(NMFeagiSimpleNeurogenesisStorageQuanted<FIQ>),
    FeagiAdvanced(NMFeagiAdvancedNeurogenesisStorageQuanted<FIQ>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum CTStorageOrGeneratorProcessingInterconnectModeled<FIQ: FeagiIndexQuantization> {
    FeagiSimple(NMFeagiSimpleNeurogenesisStorageOrGeneratorQuanted<FIQ>),
    FeagiAdvanced(NMFeagiAdvancedNeurogenesisStorageOrGeneratorQuanted<FIQ>),
}


 */







//endregion


// TODO this is very noisy in this file, yes, but these will be migrated to macros eventually





//region CorticalIdentifier

#[repr(C, u8)]
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum CorticalIdentifierProcessing {
    Interconnect(CorticalIdentifierInner<<CTFoundationPower as CorticalType>::InnerCorticalIDGenerator>)
}

impl CorticalIdentifierProcessing {
    /// Must be called from `CorticalIdentifier::try_from_bytes` as it assumes it called the
    /// correct inner!
    pub(crate) fn try_from_cortical_identifier_bytes(bytes: [u8; 8]) -> Result<Self, CorticalIdentifierParseError>
    {
        debug_assert_eq!(bytes[0] % 128, 1);
        Ok(
            match bytes[1] {
                0 => {
                    let [_, _, last_six @ ..] = bytes;
                    let inner = CorticalIdentifierInner::try_new_from_bytes(last_six)?;
                    CorticalIdentifierProcessing::Interconnect(inner)
                },
                err => {
                    return Err(CorticalIdentifierParseError::InvalidCorticalSubtype(err))
                }
            }
        )
    }
}

//endregion



