use serde::{Deserialize, Serialize};
use crate::basis::cortical_area::{CorticalType};
use feagi_connectome::feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
use crate::basis::cortical_area::cortical_identifiers::{CorticalIdentifierInner, CorticalIdentifierInnerEncoderRandom, CorticalIdentifierParseError};
use crate::basis::cortical_area::processing::interconnect::{CTProcessingInterconnect, CTProcessingInterconnectNMModeled, CTProcessingInterconnectNMModeledGenerator};
#[cfg(feature = "alloc")]
use crate::basis::cortical_area::processing::interconnect::{CTProcessingInterconnectNMModeledStorage, CTProcessingInterconnectNMModeledStorageOrGenerator};


//region Cortical Subtype

/// Cortical areas for general processing, typically between inputs and outputs
pub trait CorticalTypeProcessing: CorticalType<
    InnerCorticalIDGenerator=CorticalIdentifierInnerEncoderRandom
    // Not locked to any specific layout
>
{}

/// The most common cortical type for general processing between inputs and outputs
pub mod interconnect {
    use serde::{Deserialize, Serialize};
    use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::{NeuronLayoutOf, NeuronLayoutRestrictor};
    use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::voxel::{NeuronLayoutRestrictorVoxelUnbound, NeuronLayoutTagVoxel};
    use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
    use feagi_connectome::feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
    use feagi_connectome::neuron_models::models::feagi_advanced::quantizations::{NMFeagiAdvancedQuantKind, NMFeagiAdvancedQuanted};
    use feagi_connectome::neuron_models::models::feagi_simple::quantizations::{NMFeagiSimpleQuantKind, NMFeagiSimpleQuanted};
    use feagi_connectome::neuron_models::models::kind::NMKind;
    use feagi_connectome::neuron_models::models::{NMModeled, NMNeurogenesisGenerator, NMNeurogenesisValidatedGenerator};
    #[cfg(feature = "alloc")]
    use feagi_connectome::neuron_models::models::{NMNeurogenesisStorage, NMNeurogenesisStorageOrGenerator, NMNeurogenesisValidatedStorage, NMNeurogenesisValidatedStorageOrGenerator};
    use crate::basis::cortical_area::cortical_identifiers::CorticalIdentifierInnerEncoderRandom;
    use crate::basis::cortical_area::processing::CorticalTypeProcessing;
    use crate::basis::cortical_area::{CorticalAllowedRegions, CorticalType};

    //region CorticalType

    /// All the neuron models supported by cortical type Processing interconnect
    #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub enum CTProcessingInterconnectNMModeled {
        FeagiAdvanced(NMFeagiAdvancedQuantKind),
        FeagiSimple(NMFeagiSimpleQuantKind),
    }

    impl From<CTProcessingInterconnectNMModeled> for NMKind {
        fn from(value: CTProcessingInterconnectNMModeled) -> Self {
            match value {
                CTProcessingInterconnectNMModeled::FeagiAdvanced(m) => NMKind::FeagiAdvanced(m),
                CTProcessingInterconnectNMModeled::FeagiSimple(m) => NMKind::FeagiSimple(m),
            }
        }
    }

    /// Cortical Type for general processing
    pub struct CTProcessingInterconnect(CTProcessingInterconnectNMModeled);

    impl CorticalType for CTProcessingInterconnect {
        type NeuronLayout = NeuronLayoutTagVoxel;

        type NeuronLayoutRestrictor<FIQ: FeagiIndexQuantization> = NeuronLayoutRestrictorVoxelUnbound<FIQ>;
        type InnerCorticalIDGenerator = CorticalIdentifierInnerEncoderRandom;
        const ALLOWED_REGIONS: CorticalAllowedRegions = CorticalAllowedRegions::InnerOnly;

        type PossibleNeuronModelKindEnum = CTProcessingInterconnectNMModeled;

        type PossibleNeuronModelNeurogenesisGenerator<FIQ: FeagiIndexQuantization> = CTProcessingInterconnectNMModeledGenerator<FIQ>;

        fn into_neuron_model_enum(self) -> Self::PossibleNeuronModelKindEnum { self.0 }
    }

    impl CorticalTypeProcessing for CTProcessingInterconnect {}

    //endregion

    //region NeuronModel

    /// The NMs / Quants allowed for this Cortical Type
    pub trait CTProcessingInterconnectNM: NeuronModel
    {
        /// Spawns the generator, effectively the default parameters for creating a cortical area
        fn spawn_neurogenesis_generator<
            FIQ: FeagiIndexQuantization,
            Res: NeuronLayoutRestrictor<FIQ, NeuronLayout=NeuronLayoutOf<FIQ, Self>>
        >() -> Self::NeurogenesisByGenerator<FIQ>;

        /// [`Self::spawn_neurogenesis_generator`], wrapped in this cortical type's enum
        fn spawn_neurogenesis_generator_wrapped<
            FIQ: FeagiIndexQuantization,
            Res: NeuronLayoutRestrictor<FIQ, NeuronLayout=NeuronLayoutOf<FIQ, Self>>
        >() -> CTProcessingInterconnectNMModeledGenerator<FIQ>;
    }

    /// Default dimensions to use for this cortical type
    const DEFAULT_DIMENSIONS: [usize; 4] = [10, 10, 10, 1]; // TODO direct const of DimensionalCorticalAreaDimensions

    // Neuron Model: Feagi Advanced
    pub mod feagi_advanced {
        use feagi_basis::neurons::{CorticalAreaNeuronPotential, DimensionalCorticalAreaDimensions};
        use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::model_data::NeuronModelUniversalCorticalProperties;
        use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::{NeuronLayoutOf, NeuronLayoutRestrictor};
        use feagi_connectome::feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
        use feagi_connectome::neuron_models::models::feagi_advanced::wrapped::Period;
        use feagi_connectome::neuron_models::models::feagi_advanced::{NMFeagiAdvancedDataCorticalProperties, NMFeagiAdvancedDataCorticalVariance, NMFeagiAdvancedDataNeuronInternal};
        use feagi_connectome::neuron_models::models::feagi_advanced::quantizations::NMFeagiAdvancedQuanted;
        use feagi_connectome::neuron_models::models::feagi_advanced::quantizations::quant_standard_32_bit::NMFeagiAdvancedQuantStandard32Bit;
        use crate::basis::cortical_area::processing::interconnect::{CTProcessingInterconnectNM, CTProcessingInterconnectNMModeledGenerator, DEFAULT_DIMENSIONS};

        impl CTProcessingInterconnectNM for NMFeagiAdvancedQuantStandard32Bit {
            fn spawn_neurogenesis_generator<FIQ: FeagiIndexQuantization, Res: NeuronLayoutRestrictor<FIQ, NeuronLayout=NeuronLayoutOf<FIQ, Self>>>()
                -> Self::NeurogenesisByGenerator<FIQ> {
                Self::NeurogenesisByGenerator {
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

            fn spawn_neurogenesis_generator_wrapped<FIQ: FeagiIndexQuantization, Res: NeuronLayoutRestrictor<FIQ, NeuronLayout=NeuronLayoutOf<FIQ, Self>>>()
                -> CTProcessingInterconnectNMModeledGenerator<FIQ> {
                CTProcessingInterconnectNMModeledGenerator::FeagiAdvanced(NMFeagiAdvancedQuanted::Standard32Bit(Self::spawn_neurogenesis_generator::<FIQ, Res>()))
            }
        }
    }

    // Neuron Model: Feagi Simple
    pub mod feagi_simple {
        use feagi_basis::neurons::{CorticalAreaNeuronPotential, DimensionalCorticalAreaDimensions};
        use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::model_data::NeuronModelUniversalCorticalProperties;
        use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::{NeuronLayoutOf, NeuronLayoutRestrictor};
        use feagi_connectome::feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
        use feagi_connectome::neuron_models::models::feagi_simple::wrapped::Period;
        use feagi_connectome::neuron_models::models::feagi_simple::{NMFeagiSimpleDataCorticalProperties, NMFeagiSimpleDataCorticalVariance, NMFeagiSimpleDataNeuronInternal};
        use feagi_connectome::neuron_models::models::feagi_simple::quantizations::NMFeagiSimpleQuanted;
        use feagi_connectome::neuron_models::models::feagi_simple::quantizations::quant_standard_32_bit::NMFeagiSimpleQuantStandard32Bit;
        use crate::basis::cortical_area::processing::interconnect::{CTProcessingInterconnectNM, CTProcessingInterconnectNMModeledGenerator, DEFAULT_DIMENSIONS};

        impl CTProcessingInterconnectNM for NMFeagiSimpleQuantStandard32Bit {
            fn spawn_neurogenesis_generator<FIQ: FeagiIndexQuantization, Res: NeuronLayoutRestrictor<FIQ, NeuronLayout=NeuronLayoutOf<FIQ, Self>>>()
                -> Self::NeurogenesisByGenerator<FIQ> {
                Self::NeurogenesisByGenerator {
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

            fn spawn_neurogenesis_generator_wrapped<FIQ: FeagiIndexQuantization, Res: NeuronLayoutRestrictor<FIQ, NeuronLayout=NeuronLayoutOf<FIQ, Self>>>()
                -> CTProcessingInterconnectNMModeledGenerator<FIQ> {
                CTProcessingInterconnectNMModeledGenerator::FeagiSimple(NMFeagiSimpleQuanted::Standard32Bit(Self::spawn_neurogenesis_generator::<FIQ, Res>()))
            }
        }
    }

    //endregion

    //region CT Interconnect Other Enums

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(bound(
        serialize = "NMFeagiAdvancedQuanted<NMNeurogenesisGenerator<FIQ>>: Serialize, NMFeagiSimpleQuanted<NMNeurogenesisGenerator<FIQ>>: Serialize",
        deserialize = "NMFeagiAdvancedQuanted<NMNeurogenesisGenerator<FIQ>>: Deserialize<'de>, NMFeagiSimpleQuanted<NMNeurogenesisGenerator<FIQ>>: Deserialize<'de>"
    ))]
    pub enum CTProcessingInterconnectNMModeledGenerator<FIQ: FeagiIndexQuantization> {
        FeagiAdvanced(NMFeagiAdvancedQuanted<NMNeurogenesisGenerator<FIQ>>),
        FeagiSimple(NMFeagiSimpleQuanted<NMNeurogenesisGenerator<FIQ>>),
    }

    impl<FIQ: FeagiIndexQuantization> From<CTProcessingInterconnectNMModeledGenerator<FIQ>> for NMModeled<NMNeurogenesisGenerator<FIQ>> {
        fn from(value: CTProcessingInterconnectNMModeledGenerator<FIQ>) -> Self {
            match value {
                CTProcessingInterconnectNMModeledGenerator::FeagiAdvanced(v) => NMModeled::FeagiAdvanced(v),
                CTProcessingInterconnectNMModeledGenerator::FeagiSimple(v) => NMModeled::FeagiSimple(v),
            }
        }
    }

    #[derive(Debug, Clone)]
    pub enum CTProcessingInterconnectNMModeledValidatedGenerator<FIQ: FeagiIndexQuantization> {
        FeagiAdvanced(NMFeagiAdvancedQuanted<NMNeurogenesisValidatedGenerator<FIQ>>),
        FeagiSimple(NMFeagiSimpleQuanted<NMNeurogenesisValidatedGenerator<FIQ>>),
    }



    #[cfg(feature = "alloc")]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(bound(
        serialize = "NMFeagiAdvancedQuanted<NMNeurogenesisStorage<FIQ>>: Serialize, NMFeagiSimpleQuanted<NMNeurogenesisStorage<FIQ>>: Serialize",
        deserialize = "NMFeagiAdvancedQuanted<NMNeurogenesisStorage<FIQ>>: Deserialize<'de>, NMFeagiSimpleQuanted<NMNeurogenesisStorage<FIQ>>: Deserialize<'de>"
    ))]
    pub enum CTProcessingInterconnectNMModeledStorage<FIQ: FeagiIndexQuantization> {
        FeagiAdvanced(NMFeagiAdvancedQuanted<NMNeurogenesisStorage<FIQ>>),
        FeagiSimple(NMFeagiSimpleQuanted<NMNeurogenesisStorage<FIQ>>),
    }

    #[cfg(feature = "alloc")]
    #[derive(Debug, Clone)]
    pub enum CTProcessingInterconnectNMModeledValidatedStorage<FIQ: FeagiIndexQuantization> {
        FeagiAdvanced(NMFeagiAdvancedQuanted<NMNeurogenesisValidatedStorage<FIQ>>),
        FeagiSimple(NMFeagiSimpleQuanted<NMNeurogenesisValidatedStorage<FIQ>>),
    }



    #[cfg(feature = "alloc")]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(bound(
        serialize = "NMFeagiAdvancedQuanted<NMNeurogenesisStorageOrGenerator<FIQ>>: Serialize, NMFeagiSimpleQuanted<NMNeurogenesisStorageOrGenerator<FIQ>>: Serialize",
        deserialize = "NMFeagiAdvancedQuanted<NMNeurogenesisStorageOrGenerator<FIQ>>: Deserialize<'de>, NMFeagiSimpleQuanted<NMNeurogenesisStorageOrGenerator<FIQ>>: Deserialize<'de>"
    ))]
    pub enum CTProcessingInterconnectNMModeledStorageOrGenerator<FIQ: FeagiIndexQuantization> {
        FeagiAdvanced(NMFeagiAdvancedQuanted<NMNeurogenesisStorageOrGenerator<FIQ>>),
        FeagiSimple(NMFeagiSimpleQuanted<NMNeurogenesisStorageOrGenerator<FIQ>>),
    }

    #[cfg(feature = "alloc")]
    #[derive(Debug, Clone)]
    pub enum CTProcessingInterconnectNMModeledValidatedStorageOrGenerator<FIQ: FeagiIndexQuantization> {
        FeagiAdvanced(NMFeagiAdvancedQuanted<NMNeurogenesisValidatedStorageOrGenerator<FIQ>>),
        FeagiSimple(NMFeagiSimpleQuanted<NMNeurogenesisValidatedStorageOrGenerator<FIQ>>),
    }

    //endregion

}




#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CTProcessingSubtypedKind {
    Interconnect(CTProcessingInterconnectNMModeled),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(serialize = "CTProcessingInterconnectNMModeledGenerator<FIQ>: Serialize", deserialize = "CTProcessingInterconnectNMModeledGenerator<FIQ>: Deserialize<'de>"))]
pub enum CTProcessingSubtypedGeneratorPrecursor<FIQ: FeagiIndexQuantization> {
    Interconnect(CTProcessingInterconnectNMModeledGenerator<FIQ>)
}

#[cfg(feature = "alloc")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(serialize = "CTProcessingInterconnectNMModeledStorage<FIQ>: Serialize", deserialize = "CTProcessingInterconnectNMModeledStorage<FIQ>: Deserialize<'de>"))]
pub enum CTProcessingSubtypedStorage<FIQ: FeagiIndexQuantization> {
    Interconnect(CTProcessingInterconnectNMModeledStorage<FIQ>)
}

#[cfg(feature = "alloc")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(serialize = "CTProcessingInterconnectNMModeledStorageOrGenerator<FIQ>: Serialize", deserialize = "CTProcessingInterconnectNMModeledStorageOrGenerator<FIQ>: Deserialize<'de>"))]
pub enum CTProcessingSubtypedStorageOrGeneratorPrecursor<FIQ: FeagiIndexQuantization> {
    Interconnect(CTProcessingInterconnectNMModeledStorageOrGenerator<FIQ>)
}

//endregion


// TODO this is very noisy in this file, yes, but these will be migrated to macros eventually




//region CorticalIdentifier

#[repr(C, u8)]
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum CorticalIdentifierProcessing {
    Interconnect(CorticalIdentifierInner<<CTProcessingInterconnect as CorticalType>::InnerCorticalIDGenerator>)
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
