use crate::basis::cortical_area::cortical_identifiers::inner::CorticalIdentifierInner;
use crate::basis::cortical_area::cortical_identifiers::{CorticalIdentifierParseError};
use crate::basis::cortical_area::processing::interconnect::{CTProcessingInterconnect, CTProcessingInterconnectNMModeledGenerator};
#[cfg(feature = "alloc")]
use crate::basis::cortical_area::processing::interconnect::{CTProcessingInterconnectNMModeledStorage, CTProcessingInterconnectNMModeledStorageOrGenerator};
#[cfg(feature = "alloc")]
use crate::basis::cortical_area::CorticalTypeAreaSpawningParameters;
use crate::basis::cortical_area::CorticalType;
use feagi_connectome::connectome_basis::quantization_levels::FeagiIndexQuantization;
use serde::{Deserialize, Serialize};
use crate::basis::cortical_area::cortical_identifiers::inner::inner_random::CorticalIdentifierInnerEncoderRandom;
use crate::basis::cortical_area::cortical_identifiers::inner::random_byte_generator::CorticalIDInnerRNG;
use crate::basis::cortical_area::spawning::{CorticalTypeSpawningRequest, CorticalTypeSpawningValidated};
//region Cortical Subtypes

/// Cortical areas for general processing, typically between inputs and outputs
pub trait CorticalTypeProcessing: CorticalType<
    InnerCorticalIDEncoder=CorticalIdentifierInnerEncoderRandom
    // Not locked to any specific layout
>
{}

/// The most common cortical type for general processing between inputs and outputs
pub mod interconnect {
    use crate::basis::cortical_area::processing::CorticalTypeProcessing;
    use crate::basis::cortical_area::{CorticalAllowedRegions, CorticalType};
    use feagi_connectome::connectome_basis::neuron_model::neuron_layout::voxel::{NeuronLayoutRestrictorVoxelUnbound, NeuronLayoutTagVoxel};
    use feagi_connectome::connectome_basis::neuron_model::neuron_layout::{NeuronLayoutOf, NeuronLayoutRestrictor};
    use feagi_connectome::connectome_basis::neuron_model::NeuronModel;
    use feagi_connectome::connectome_basis::quantization_levels::FeagiIndexQuantization;
    use feagi_connectome::connectome_models::neuron_models::models::feagi_advanced::quantizations::{NMFeagiAdvancedQuantKind, NMFeagiAdvancedQuanted};
    use feagi_connectome::connectome_models::neuron_models::models::feagi_simple::quantizations::{NMFeagiSimpleQuantKind, NMFeagiSimpleQuanted};
    use feagi_connectome::connectome_models::neuron_models::models::kind::NMKind;
    use feagi_connectome::connectome_models::neuron_models::models::{NMModeled, NMNeurogenesisGenerator};
    #[cfg(feature = "alloc")]
    use feagi_connectome::connectome_models::neuron_models::models::{NMNeurogenesisStorage, NMNeurogenesisStorageOrGenerator};
    #[cfg(feature = "alloc")]
    use feagi_connectome::connectome_basis::neuron_model::neurogenesis::by_generator::NeurogenesisOtherDataForStorageOf;
    #[cfg(feature = "alloc")]
    use feagi_connectome::connectome_basis::neuron_model::neurogenesis::NeurogenesisStorageOrGenerator;
    #[cfg(feature = "alloc")]
    use feagi_connectome::connectome_basis::neuron_model::neurogenesis::from_storage::NeurogenesisByStorage;
    #[cfg(feature = "alloc")]
    use feagi_connectome::connectome_models::neuron_models::models::feagi_advanced::quantizations::quant_standard_32_bit::NMFeagiAdvancedQuantStandard32Bit;
    #[cfg(feature = "alloc")]
    use feagi_connectome::connectome_models::neuron_models::models::feagi_simple::quantizations::quant_standard_32_bit::NMFeagiSimpleQuantStandard32Bit;
    use serde::{Deserialize, Serialize};
    use crate::basis::cortical_area::cortical_identifiers::{CorticalIdentifier, FauxCorticalIdentifier};
    use crate::basis::cortical_area::cortical_identifiers::inner::CorticalIdentifierInner;
    use crate::basis::cortical_area::cortical_identifiers::inner::inner_random::CorticalIdentifierInnerEncoderRandom;
    use crate::basis::cortical_area::processing::CorticalIdentifierProcessing;
    //region CorticalType

    /// Cortical Type for general processing. Each variant is a neuron model (and its quantization)
    /// supported by this cortical type
    #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub enum CTProcessingInterconnect {
        FeagiAdvanced(NMFeagiAdvancedQuantKind),
        FeagiSimple(NMFeagiSimpleQuantKind),
    }

    impl From<CTProcessingInterconnect> for NMKind {
        fn from(value: CTProcessingInterconnect) -> Self {
            match value {
                CTProcessingInterconnect::FeagiAdvanced(m) => NMKind::FeagiAdvanced(m),
                CTProcessingInterconnect::FeagiSimple(m) => NMKind::FeagiSimple(m),
            }
        }
    }

    impl CorticalType for CTProcessingInterconnect {
        type NeuronLayout = NeuronLayoutTagVoxel;

        type NeuronLayoutRestrictor<FIQ: FeagiIndexQuantization> = NeuronLayoutRestrictorVoxelUnbound<FIQ>;
        type InnerCorticalIDEncoder = CorticalIdentifierInnerEncoderRandom;
        const ALLOWED_REGIONS: CorticalAllowedRegions = CorticalAllowedRegions::InnerOnly;

        type NeuronModelNeurogenesisGenerator<FIQ: FeagiIndexQuantization> = CTProcessingInterconnectNMModeledGenerator<FIQ>;
        #[cfg(feature = "alloc")]
        type NeuronModelNeuroGenesisStorageOrGenerator<FIQ: FeagiIndexQuantization> = CTProcessingInterconnectNMModeledStorageOrGenerator<FIQ>;

        fn wrap_cortical_identifier_inner(inner: CorticalIdentifierInner<Self::InnerCorticalIDEncoder>) -> CorticalIdentifier {
            CorticalIdentifier::Processing(CorticalIdentifierProcessing::Interconnect(inner))
        }

        fn wrap_faux_cortical_identifier_inner(inner: CorticalIdentifierInner<Self::InnerCorticalIDEncoder>) -> FauxCorticalIdentifier {
            FauxCorticalIdentifier::Processing(CorticalIdentifierProcessing::Interconnect(inner))
        }

        /// Storage is converted using this cortical type's default neuron internal data
        #[cfg(feature = "alloc")]
        fn copy_storage_or_generator_as_generator<FIQ: FeagiIndexQuantization>(neurogenesis_model_enum: &Self::NeuronModelNeuroGenesisStorageOrGenerator<FIQ>) -> Self::NeuronModelNeurogenesisGenerator<FIQ> {
            match neurogenesis_model_enum {
                CTProcessingInterconnectNMModeledStorageOrGenerator::FeagiAdvanced(quant_enum) => {
                    match quant_enum {
                        NMFeagiAdvancedQuanted::Standard32Bit(storage_or_generator) => {
                            let defaults = NMFeagiAdvancedQuantStandard32Bit::spawn_neurogenesis_generator::<FIQ, Self::NeuronLayoutRestrictor<FIQ>>();
                            NMFeagiAdvancedQuantStandard32Bit::copy_generator_from_storage_or_generator_wrapped(storage_or_generator, defaults.neuron_internal)
                        }
                    }
                }
                CTProcessingInterconnectNMModeledStorageOrGenerator::FeagiSimple(quant_enum) => {
                    match quant_enum {
                        NMFeagiSimpleQuanted::Standard32Bit(storage_or_generator) => {
                            let defaults = NMFeagiSimpleQuantStandard32Bit::spawn_neurogenesis_generator::<FIQ, Self::NeuronLayoutRestrictor<FIQ>>();
                            NMFeagiSimpleQuantStandard32Bit::copy_generator_from_storage_or_generator_wrapped(storage_or_generator, defaults.neuron_internal)
                        }
                    }
                }
            }
        }
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

        /// Wraps a generator of this neuron model in this cortical type's enum. Per model, since
        /// each neuron model is a different variant
        fn wrap_generator<FIQ: FeagiIndexQuantization>(
            generator: Self::NeurogenesisByGenerator<FIQ>
        ) -> CTProcessingInterconnectNMModeledGenerator<FIQ>;

        //region Default Impls

        /// [`Self::spawn_neurogenesis_generator`], wrapped in this cortical type's enum
        fn spawn_neurogenesis_generator_wrapped<
            FIQ: FeagiIndexQuantization,
            Res: NeuronLayoutRestrictor<FIQ, NeuronLayout=NeuronLayoutOf<FIQ, Self>>
        >() -> CTProcessingInterconnectNMModeledGenerator<FIQ> {
            Self::wrap_generator(Self::spawn_neurogenesis_generator::<FIQ, Res>())
        }

        /// [`NeurogenesisByStorage::to_generator`], wrapped in this cortical type's enum
        #[cfg(feature = "alloc")]
        fn spawn_neurogenesis_generator_from_storage_wrapped<
            FIQ: FeagiIndexQuantization,
        >(
            storage: &NeurogenesisByStorage<FIQ, Self>,
            other_data: NeurogenesisOtherDataForStorageOf<FIQ, Self>,
        ) -> CTProcessingInterconnectNMModeledGenerator<FIQ> {
            Self::wrap_generator(storage.to_generator(other_data))
        }

        /// [`NeurogenesisStorageOrGenerator::copy_as_generator`], wrapped in this cortical type's enum
        #[cfg(feature = "alloc")]
        fn copy_generator_from_storage_or_generator_wrapped<
            FIQ: FeagiIndexQuantization
        >(
            storage_or_generator: &NeurogenesisStorageOrGenerator<FIQ, Self>,
            other_data: NeurogenesisOtherDataForStorageOf<FIQ, Self>,
        ) -> CTProcessingInterconnectNMModeledGenerator<FIQ> {
            Self::wrap_generator(storage_or_generator.copy_as_generator(other_data))
        }

        //endregion
    }

    /// Default dimensions to use for this cortical type
    const DEFAULT_DIMENSIONS: [usize; 4] = [10, 10, 10, 1]; // TODO direct const of DimensionalCorticalAreaDimensions

    // Neuron Model: Feagi Advanced
    pub mod feagi_advanced {
        use crate::basis::cortical_area::processing::interconnect::{CTProcessingInterconnectNM, CTProcessingInterconnectNMModeledGenerator, DEFAULT_DIMENSIONS};
        use feagi_basis::neurons::{CorticalAreaNeuronPotential, DimensionalCorticalAreaDimensions};
        use feagi_connectome::connectome_basis::neuron_model::model_data::NeuronModelUniversalCorticalProperties;
        use feagi_connectome::connectome_basis::neuron_model::neuron_layout::{NeuronLayoutOf, NeuronLayoutRestrictor};
        use feagi_connectome::connectome_basis::quantization_levels::FeagiIndexQuantization;
        use feagi_connectome::connectome_models::neuron_models::models::feagi_advanced::quantizations::quant_standard_32_bit::NMFeagiAdvancedQuantStandard32Bit;
        use feagi_connectome::connectome_models::neuron_models::models::feagi_advanced::quantizations::NMFeagiAdvancedQuanted;
        use feagi_connectome::connectome_models::neuron_models::models::feagi_advanced::wrapped::Period;
        use feagi_connectome::connectome_models::neuron_models::models::feagi_advanced::{NMFeagiAdvancedDataCorticalProperties, NMFeagiAdvancedDataCorticalVariance, NMFeagiAdvancedDataNeuronInternal};

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

            fn wrap_generator<FIQ: FeagiIndexQuantization>(generator: Self::NeurogenesisByGenerator<FIQ>)
                -> CTProcessingInterconnectNMModeledGenerator<FIQ> {
                CTProcessingInterconnectNMModeledGenerator::FeagiAdvanced(NMFeagiAdvancedQuanted::Standard32Bit(generator))
            }
        }
    }

    // Neuron Model: Feagi Simple
    pub mod feagi_simple {
        use crate::basis::cortical_area::processing::interconnect::{CTProcessingInterconnectNM, CTProcessingInterconnectNMModeledGenerator, DEFAULT_DIMENSIONS};
        use feagi_basis::neurons::{CorticalAreaNeuronPotential, DimensionalCorticalAreaDimensions};
        use feagi_connectome::connectome_basis::neuron_model::model_data::NeuronModelUniversalCorticalProperties;
        use feagi_connectome::connectome_basis::neuron_model::neuron_layout::{NeuronLayoutOf, NeuronLayoutRestrictor};
        use feagi_connectome::connectome_basis::quantization_levels::FeagiIndexQuantization;
        use feagi_connectome::connectome_models::neuron_models::models::feagi_simple::quantizations::quant_standard_32_bit::NMFeagiSimpleQuantStandard32Bit;
        use feagi_connectome::connectome_models::neuron_models::models::feagi_simple::quantizations::NMFeagiSimpleQuanted;
        use feagi_connectome::connectome_models::neuron_models::models::feagi_simple::wrapped::Period;
        use feagi_connectome::connectome_models::neuron_models::models::feagi_simple::{NMFeagiSimpleDataCorticalProperties, NMFeagiSimpleDataCorticalVariance, NMFeagiSimpleDataNeuronInternal};

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

            fn wrap_generator<FIQ: FeagiIndexQuantization>(generator: Self::NeurogenesisByGenerator<FIQ>)
                -> CTProcessingInterconnectNMModeledGenerator<FIQ> {
                CTProcessingInterconnectNMModeledGenerator::FeagiSimple(NMFeagiSimpleQuanted::Standard32Bit(generator))
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
    impl<FIQ: FeagiIndexQuantization> From<CTProcessingInterconnectNMModeledStorageOrGenerator<FIQ>> for NMModeled<NMNeurogenesisStorageOrGenerator<FIQ>> {
        fn from(value: CTProcessingInterconnectNMModeledStorageOrGenerator<FIQ>) -> Self {
            match value {
                CTProcessingInterconnectNMModeledStorageOrGenerator::FeagiAdvanced(sog) => NMModeled::FeagiAdvanced(sog),
                CTProcessingInterconnectNMModeledStorageOrGenerator::FeagiSimple(sog) => NMModeled::FeagiSimple(sog),
            }
        }
    }


    //endregion

}




#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CTProcessingSubtypedKind {
    Interconnect(CTProcessingInterconnect),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(serialize = "CTProcessingInterconnectNMModeledGenerator<FIQ>: Serialize", deserialize = "CTProcessingInterconnectNMModeledGenerator<FIQ>: Deserialize<'de>"))]
pub enum CTProcessingSubtypedGenerator<FIQ: FeagiIndexQuantization> {
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
pub enum CTProcessingSubtypedStorageOrGenerator<FIQ: FeagiIndexQuantization> {
    Interconnect(CTProcessingInterconnectNMModeledStorageOrGenerator<FIQ>)
}

#[cfg(feature = "alloc")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum CTProcessingSubtypedSpawningRequest<FIQ: FeagiIndexQuantization> {
    Interconnect(CorticalTypeSpawningRequest<FIQ, CTProcessingInterconnect>)
}

#[cfg(feature = "alloc")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub(crate) enum CTProcessingSubtypedSpawningValidated<FIQ: FeagiIndexQuantization> {
    Interconnect(CorticalTypeSpawningValidated<FIQ, CTProcessingInterconnect>)
}

//endregion

// TODO this is very noisy in this file, yes, but these will be migrated to macros eventually


//region CorticalIdentifier

#[repr(C, u8)]
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum CorticalIdentifierProcessing {
    Interconnect(CorticalIdentifierInner<<CTProcessingInterconnect as CorticalType>::InnerCorticalIDEncoder>)
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

    /// Rerolls the inner cortical ID RNG portion of the ID, keeping the same data
    pub fn reroll_inner<RNG: CorticalIDInnerRNG>(self, rng: &RNG) -> Self {
        match self {
            CorticalIdentifierProcessing::Interconnect(inner) => {
                CorticalIdentifierProcessing::Interconnect(inner.reroll_inner(rng))
            }
        }
    }
}

//endregion
