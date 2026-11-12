use serde::{Deserialize, Serialize};
use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::{NeuronLayout, NeuronLayoutRestrictor, NeuronLayoutTag};
use crate::basis::cortical_area::{CorticalSubTypeUsesDefaultCorticalGenerator, CorticalType};
use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome::feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
use crate::basis::cortical_area::cortical_identifiers::CorticalIdentifierParseError;



//region Cortical Subtype

/// This cortical type is for
pub trait CorticalTypeProcessing<NM>: CorticalType<NM, IdentifierInner=[u8; 6]> // uses 6 bytes as cortical ID inner
where
    NM: NeuronModel<NeuronLayoutTag=Self::NeuronLayout>
{}

/// The most common cortical type for general processing between inputs and outputs
pub mod interconnect {

    //region Supported Neuron Models for Interconnect

    use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::{NeuronLayoutOf, NeuronLayoutRestrictor};
    use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
    use feagi_connectome::feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
    use feagi_connectome::neuron_models::models::feagi_advanced::quantizations::NMFeagiAdvancedQuantedKind;
    use feagi_connectome::neuron_models::models::feagi_simple::quantizations::NMFeagiSimpleQuantedKind;

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
        use feagi_basis::neurons::{CorticalAreaNeuronPotential, DimensionalCorticalAreaDimensions};
        use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::model_data::NeuronModelUniversalCorticalProperties;
        use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::{NeuronLayoutOf, NeuronLayoutRestrictor};
        use feagi_connectome::feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
        use feagi_connectome::neuron_models::models::feagi_advanced::{NMFeagiAdvancedDataCorticalProperties, NMFeagiAdvancedDataCorticalVariance, NMFeagiAdvancedDataNeuronInternal};
        use feagi_connectome::neuron_models::models::feagi_advanced::quantizations::{NMFeagiAdvancedQuantStandard32Bit, NMFeagiAdvancedQuantedKind};
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

        pub enum CTProcessingInterconnectKindModeled<FIQ: FeagiIndexQuantization> {
            Standard32Bit(NMFeagiAdvancedQuantedKind)
        }
        
        pub enum CTProcessingInterconnectGeneratorPrecursorModeled<FIQ: FeagiIndexQuantization> {
            Standard32Bit(NMFeagiAdvancedQuantStandard32Bit::NeurogenesisGeneratorPrecursor<FIQ>),
        }
        

        
        


    }

    pub mod feagi_simple {
        use feagi_basis::neurons::{CorticalAreaNeuronPotential, DimensionalCorticalAreaDimensions};
        use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::model_data::NeuronModelUniversalCorticalProperties;
        use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::{NeuronLayoutOf, NeuronLayoutRestrictor};
        use feagi_connectome::feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
        use feagi_connectome::neuron_models::models::feagi_simple::quantizations::NMFeagiSimpleQuantStandard32Bit;
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

        
    }

    
    
    
    pub enum CTProcessingInterconnectNMModeledKind {
        FeagiAdvanced(NMFeagiAdvancedQuantedKind),
        FeagiSimple(NMFeagiSimpleQuantedKind),
    }


}





//region Enum NMForCTProcessingInterconnect



//endregion






//region FeagiSimple


//endregion



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

//endregion



//endregion


//region Enums


//endregion

//endregion




#[repr(u8)]
#[derive(Copy, Clone, Debug, Hash)]
pub enum CorticalTypeProcessingSubtypeKind {
    Interconnect = 0,
}

//region Cortical Subtype Enums

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum CTGeneratorProcessingSubTyped<FIQ: FeagiIndexQuantization>
{
    Interconnect(CTGeneratorProcessingInterconnectModeled<FIQ>)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum CTStorageProcessingSubTyped<FIQ: FeagiIndexQuantization>
{
    Interconnect(CTStorageProcessingInterconnectModeled<FIQ>)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum CTStorageOrGeneratorProcessingSubTyped<FIQ: FeagiIndexQuantization>
{
    Interconnect(CTStorageOrGeneratorProcessingInterconnectModeled<FIQ>)
}

//endregion


// TODO this is very noisy in this file, yes, but these will be migrated to macros eventually





//region CorticalIdentifier

#[repr(C, u8)]
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum CorticalIdentifierProcessing {
    Interconnect([u8; 6])
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
                    CorticalIdentifierProcessing::Interconnect(last_six)
                },
                err => {
                    return Err(CorticalIdentifierParseError::InvalidCorticalSubtype(err))
                }
            }
        )
    }
}

//endregion



