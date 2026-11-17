use serde::{Deserialize, Serialize};
use crate::basis::cortical_area::{CorticalType};
#[cfg(feature = "alloc")]
use crate::basis::cortical_area::CorticalTypeAreaSpawningParameters;
use feagi_connectome::connectome_basis::neuron_model::neuron_layout::voxel::{NeuronLayoutTagVoxel};
use feagi_connectome::connectome_basis::quantization_levels::FeagiIndexQuantization;
use crate::basis::cortical_area::cortical_identifiers::{CorticalIdentifierInner, CorticalIdentifierInnerEncoderRandom, CorticalIdentifierParseError};
use crate::basis::cortical_area::foundation::power::{CTFoundationPower, CTFoundationPowerNMModeledGenerator};
#[cfg(feature = "alloc")]
use crate::basis::cortical_area::foundation::power::{CTFoundationPowerNMModeledStorage, CTFoundationPowerNMModeledStorageOrGenerator};


//region Cortical Subtype

/// These are the core cortical areas of FEAGI, that typically have some unique functionality
/// that are highly specific and possibly integrate with underlying systems. This Cortical Type
/// is unique in that pretty much every subtype has a specific single neuron model for it.
pub trait CorticalTypeFoundation: CorticalType<
    NeuronLayout=NeuronLayoutTagVoxel, // Locked to voxel layout
    InnerCorticalIDGenerator=CorticalIdentifierInnerEncoderRandom
>
{}




/// Fires every burst
pub mod power {
    use serde::{Deserialize, Serialize};
    use feagi_connectome::connectome_basis::neuron_model::neuron_layout::{NeuronLayoutOf, NeuronLayoutRestrictor};
    use feagi_connectome::connectome_basis::neuron_model::neuron_layout::voxel::{NeuronLayoutRestrictorVoxelBound, NeuronLayoutTagVoxel};
    use feagi_connectome::connectome_basis::quantization_levels::FeagiIndexQuantization;
    use feagi_connectome::connectome_models::neuron_models::models::exclusive_power::quantizations::{NMExclusivePowerQuantKind, NMExclusivePowerQuanted};
    use feagi_connectome::connectome_models::neuron_models::models::kind::NMKind;
    use feagi_connectome::connectome_models::neuron_models::models::{NMMember, NMModeled, NMNeurogenesisGenerator};
    #[cfg(feature = "alloc")]
    use feagi_connectome::connectome_models::neuron_models::models::{NMNeurogenesisStorage, NMNeurogenesisStorageOrGenerator};
    use crate::basis::cortical_area::cortical_identifiers::{CorticalIdentifierInnerEncoderRandom};
    use crate::basis::cortical_area::foundation::CorticalTypeFoundation;
    use crate::basis::cortical_area::{CorticalAllowedRegions, CorticalType};

    //region CorticalType


    /// Cortical Type that fires every burst. Each variant is a neuron model (and its quantization)
    /// supported by this cortical type
    #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub enum CTFoundationPower {
        ExclusivePower(NMExclusivePowerQuantKind)
    }

    impl From<CTFoundationPower> for NMKind {
        fn from(value: CTFoundationPower) -> Self {
            match value {
                CTFoundationPower::ExclusivePower(m) => NMKind::ExclusivePower(m),
            }
        }
    }

    impl CorticalType for CTFoundationPower {
        type NeuronLayout = NeuronLayoutTagVoxel;

        type NeuronLayoutRestrictor<FIQ: FeagiIndexQuantization> = NeuronLayoutRestrictorVoxelBound<
            FIQ, // Only allow 1x1x1x1
            1, 1, 1, 1,
            2, 2, 2, 2
        >;
        type InnerCorticalIDGenerator = CorticalIdentifierInnerEncoderRandom;
        const ALLOWED_REGIONS: CorticalAllowedRegions = CorticalAllowedRegions::RootOnly;

        type PossibleNeuronModelNeurogenesisGenerator<FIQ: FeagiIndexQuantization> = CTFoundationPowerNMModeledGenerator<FIQ>;
        #[cfg(feature = "alloc")]
        type PossibleNeuronModelNeuroGenesisStorageOrGenerator<FIQ: FeagiIndexQuantization> = CTFoundationPowerNMModeledStorageOrGenerator<FIQ>;
    }

    impl CorticalTypeFoundation for CTFoundationPower {}

    //endregion

    //region NeuronModel

    /// The NMs / Quants allowed for this Cortical Type
    pub trait CTFoundationPowerNM: NMMember
    {
        /// Spawns the generator precursor, effectively the default parameters for creating a cortical area
        fn spawn_neurogenesis_generator<
            FIQ: FeagiIndexQuantization,
            Res: NeuronLayoutRestrictor<FIQ, NeuronLayout=NeuronLayoutOf<FIQ, Self>>
        >() -> Self::NeurogenesisByGenerator<FIQ>;

        /// [`Self::spawn_neurogenesis_generator`], wrapped in this cortical type's enum
        fn spawn_neurogenesis_generator_wrapped<
            FIQ: FeagiIndexQuantization,
            Res: NeuronLayoutRestrictor<FIQ, NeuronLayout=NeuronLayoutOf<FIQ, Self>>
        >() -> CTFoundationPowerNMModeledGenerator<FIQ>
        where Self: NMMember<Quanted<NMNeurogenesisGenerator<FIQ>> = NMExclusivePowerQuanted<NMNeurogenesisGenerator<FIQ>>>
        {
            let generator = Self::spawn_neurogenesis_generator::<FIQ, Res>();
            CTFoundationPowerNMModeledGenerator::ExclusivePower(Self::wrap::<NMNeurogenesisGenerator<FIQ>>(generator))
        }
    }

    // Neuron Model: Exclusive Power
    pub mod exclusive_power {
        use feagi_basis::neurons::{CorticalAreaNeuronPotential, DimensionalCorticalAreaDimensions};
        use feagi_connectome::connectome_basis::neuron_model::model_data::NeuronModelUniversalCorticalProperties;
        use feagi_connectome::connectome_basis::neuron_model::neuron_layout::{NeuronLayoutOf, NeuronLayoutRestrictor};
        use feagi_connectome::connectome_basis::quantization_levels::FeagiIndexQuantization;
        use feagi_connectome::connectome_models::neuron_models::models::exclusive_power::{NMExclusivePowerDataCorticalInternal, NMExclusivePowerDataCorticalProperties, NMExclusivePowerDataCorticalVariance, NMExclusivePowerDataNeuronInternal};
        use feagi_connectome::connectome_models::neuron_models::models::exclusive_power::quantizations::quant_bfloat_16::NMExclusivePowerQuantBFloat16;
        use feagi_connectome::connectome_models::neuron_models::models::exclusive_power::quantizations::quant_float_16::NMExclusivePowerQuantFloat16;
        use feagi_connectome::connectome_models::neuron_models::models::exclusive_power::quantizations::quant_float_32::NMExclusivePowerQuantFloat32;
        use feagi_connectome::connectome_models::neuron_models::models::exclusive_power::quantizations::quant_float_64::NMExclusivePowerQuantFloat64;
        use crate::basis::cortical_area::foundation::power::CTFoundationPowerNM;

        /// Default dimensions to use for this cortical type. Power is always exactly a single neuron
        const DEFAULT_DIMENSIONS: [usize; 4] = [1, 1, 1, 1]; // TODO direct const of DimensionalCorticalAreaDimensions

        impl CTFoundationPowerNM for NMExclusivePowerQuantFloat16 {
            fn spawn_neurogenesis_generator<FIQ: FeagiIndexQuantization, Res: NeuronLayoutRestrictor<FIQ, NeuronLayout=NeuronLayoutOf<FIQ, Self>>>()
                -> Self::NeurogenesisByGenerator<FIQ> {
                Self::NeurogenesisByGenerator {
                    universal_properties: NeuronModelUniversalCorticalProperties {
                        cortical_level_psp: CorticalAreaNeuronPotential::QUANT_ONE,
                        is_psp_uniform: false,
                        is_psp_using_neuron_potential: false,
                    },
                    cortical_context: DimensionalCorticalAreaDimensions::new_dimensions_from_usize(DEFAULT_DIMENSIONS).unwrap(),
                    cortical_properties: NMExclusivePowerDataCorticalProperties::new(),
                    cortical_internal: NMExclusivePowerDataCorticalInternal::new(),
                    cortical_variance: NMExclusivePowerDataCorticalVariance::new(),
                    neuron_internal: NMExclusivePowerDataNeuronInternal::new(),
                }
            }
        }

        impl CTFoundationPowerNM for NMExclusivePowerQuantBFloat16 {
            fn spawn_neurogenesis_generator<FIQ: FeagiIndexQuantization, Res: NeuronLayoutRestrictor<FIQ, NeuronLayout=NeuronLayoutOf<FIQ, Self>>>()
                -> Self::NeurogenesisByGenerator<FIQ> {
                Self::NeurogenesisByGenerator {
                    universal_properties: NeuronModelUniversalCorticalProperties {
                        cortical_level_psp: CorticalAreaNeuronPotential::QUANT_ONE,
                        is_psp_uniform: false,
                        is_psp_using_neuron_potential: false,
                    },
                    cortical_context: DimensionalCorticalAreaDimensions::new_dimensions_from_usize(DEFAULT_DIMENSIONS).unwrap(),
                    cortical_properties: NMExclusivePowerDataCorticalProperties::new(),
                    cortical_internal: NMExclusivePowerDataCorticalInternal::new(),
                    cortical_variance: NMExclusivePowerDataCorticalVariance::new(),
                    neuron_internal: NMExclusivePowerDataNeuronInternal::new(),
                }
            }
        }

        impl CTFoundationPowerNM for NMExclusivePowerQuantFloat32 {
            fn spawn_neurogenesis_generator<FIQ: FeagiIndexQuantization, Res: NeuronLayoutRestrictor<FIQ, NeuronLayout=NeuronLayoutOf<FIQ, Self>>>()
                -> Self::NeurogenesisByGenerator<FIQ> {
                Self::NeurogenesisByGenerator {
                    universal_properties: NeuronModelUniversalCorticalProperties {
                        cortical_level_psp: CorticalAreaNeuronPotential::QUANT_ONE,
                        is_psp_uniform: false,
                        is_psp_using_neuron_potential: false,
                    },
                    cortical_context: DimensionalCorticalAreaDimensions::new_dimensions_from_usize(DEFAULT_DIMENSIONS).unwrap(),
                    cortical_properties: NMExclusivePowerDataCorticalProperties::new(),
                    cortical_internal: NMExclusivePowerDataCorticalInternal::new(),
                    cortical_variance: NMExclusivePowerDataCorticalVariance::new(),
                    neuron_internal: NMExclusivePowerDataNeuronInternal::new(),
                }
            }
        }

        impl CTFoundationPowerNM for NMExclusivePowerQuantFloat64 {
            fn spawn_neurogenesis_generator<FIQ: FeagiIndexQuantization, Res: NeuronLayoutRestrictor<FIQ, NeuronLayout=NeuronLayoutOf<FIQ, Self>>>()
                -> Self::NeurogenesisByGenerator<FIQ> {
                Self::NeurogenesisByGenerator {
                    universal_properties: NeuronModelUniversalCorticalProperties {
                        cortical_level_psp: CorticalAreaNeuronPotential::QUANT_ONE,
                        is_psp_uniform: false,
                        is_psp_using_neuron_potential: false,
                    },
                    cortical_context: DimensionalCorticalAreaDimensions::new_dimensions_from_usize(DEFAULT_DIMENSIONS).unwrap(),
                    cortical_properties: NMExclusivePowerDataCorticalProperties::new(),
                    cortical_internal: NMExclusivePowerDataCorticalInternal::new(),
                    cortical_variance: NMExclusivePowerDataCorticalVariance::new(),
                    neuron_internal: NMExclusivePowerDataNeuronInternal::new(),
                }
            }
        }

    }

    //endregion

    //region CT Power Other Enums

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(bound(serialize = "NMExclusivePowerQuanted<NMNeurogenesisGenerator<FIQ>>: Serialize", deserialize = "NMExclusivePowerQuanted<NMNeurogenesisGenerator<FIQ>>: Deserialize<'de>"))]
    pub enum CTFoundationPowerNMModeledGenerator<FIQ: FeagiIndexQuantization> {
        ExclusivePower(NMExclusivePowerQuanted<NMNeurogenesisGenerator<FIQ>>),
    }

    impl<FIQ: FeagiIndexQuantization> From<CTFoundationPowerNMModeledGenerator<FIQ>> for NMModeled<NMNeurogenesisGenerator<FIQ>> {
        fn from(value: CTFoundationPowerNMModeledGenerator<FIQ>) -> Self {
            match value {
                CTFoundationPowerNMModeledGenerator::ExclusivePower(v) => NMModeled::ExclusivePower(v),
            }
        }
    }

    #[cfg(feature = "alloc")]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(bound(serialize = "NMExclusivePowerQuanted<NMNeurogenesisStorage<FIQ>>: Serialize", deserialize = "NMExclusivePowerQuanted<NMNeurogenesisStorage<FIQ>>: Deserialize<'de>"))]
    pub enum CTFoundationPowerNMModeledStorage<FIQ: FeagiIndexQuantization> {
        ExclusivePower(NMExclusivePowerQuanted<NMNeurogenesisStorage<FIQ>>),
    }

    #[cfg(feature = "alloc")]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(bound(serialize = "NMExclusivePowerQuanted<NMNeurogenesisStorageOrGenerator<FIQ>>: Serialize", deserialize = "NMExclusivePowerQuanted<NMNeurogenesisStorageOrGenerator<FIQ>>: Deserialize<'de>"))]
    pub enum CTFoundationPowerNMModeledStorageOrGenerator<FIQ: FeagiIndexQuantization> {
        ExclusivePower(NMExclusivePowerQuanted<NMNeurogenesisStorageOrGenerator<FIQ>>),
    }

    #[cfg(feature = "alloc")]
    impl<FIQ: FeagiIndexQuantization> From<CTFoundationPowerNMModeledStorageOrGenerator<FIQ>> for NMModeled<NMNeurogenesisStorageOrGenerator<FIQ>> {
        fn from(value: CTFoundationPowerNMModeledStorageOrGenerator<FIQ>) -> Self {
            match value {
                CTFoundationPowerNMModeledStorageOrGenerator::ExclusivePower(sog) => {
                    NMModeled::ExclusivePower(sog)
                }
            }
        }
    }
    
    

    //endregion

}




#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CTFoundationSubtypedKind {
    Power(CTFoundationPower),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(serialize = "CTFoundationPowerNMModeledGenerator<FIQ>: Serialize", deserialize = "CTFoundationPowerNMModeledGenerator<FIQ>: Deserialize<'de>"))]
pub enum CTFoundationSubtypedGenerator<FIQ: FeagiIndexQuantization> {
    Power(CTFoundationPowerNMModeledGenerator<FIQ>)
}

#[cfg(feature = "alloc")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(serialize = "CTFoundationPowerNMModeledStorage<FIQ>: Serialize", deserialize = "CTFoundationPowerNMModeledStorage<FIQ>: Deserialize<'de>"))]
pub enum CTFoundationSubtypedStorage<FIQ: FeagiIndexQuantization> {
    Power(CTFoundationPowerNMModeledStorage<FIQ>)
}

#[cfg(feature = "alloc")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(serialize = "CTFoundationPowerNMModeledStorageOrGenerator<FIQ>: Serialize", deserialize = "CTFoundationPowerNMModeledStorageOrGenerator<FIQ>: Deserialize<'de>"))]
pub enum CTFoundationSubtypedStorageOrGenerator<FIQ: FeagiIndexQuantization> {
    Power(CTFoundationPowerNMModeledStorageOrGenerator<FIQ>)
}

#[cfg(feature = "alloc")]
pub enum CTFoundationSubtypedSpawningParameters<FIQ: FeagiIndexQuantization> {
    Power(CorticalTypeAreaSpawningParameters<FIQ, CTFoundationPower>)
}

//endregion


// TODO this is very noisy in this file, yes, but these will be migrated to macros eventually




//region CorticalIdentifier

#[repr(C, u8)]
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum CorticalIdentifierFoundation {
    Power(CorticalIdentifierInner<<CTFoundationPower as CorticalType>::InnerCorticalIDGenerator>) = 0
}

impl CorticalIdentifierFoundation {

    /// Must be called from `CorticalIdentifier::try_from_bytes` as it assumes it called the
    /// correct inner!
    pub(crate) fn try_from_cortical_identifier_bytes(bytes: [u8; 8]) -> Result<Self, CorticalIdentifierParseError>
    {
        debug_assert_eq!(bytes[0] % 128, 0);
        Ok(
            match bytes[1] {
                0 => { // Power
                    let [_, _, last_six @ ..] = bytes;

                    let inner = CorticalIdentifierInner::try_new_from_bytes(last_six)?;
                    CorticalIdentifierFoundation::Power(inner)
                },
                err => {
                    return Err(CorticalIdentifierParseError::InvalidCorticalSubtype(err))
                }
            }
        )
    }

}

//endregion
