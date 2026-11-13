use serde::{Deserialize, Serialize};
use crate::basis::cortical_area::{CorticalType};
use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::voxel::{NeuronLayoutTagVoxel};
use feagi_connectome::feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
use crate::basis::cortical_area::cortical_identifiers::{CorticalIdentifierInner, CorticalIdentifierInnerEncoder, CorticalIdentifierInnerEncoderRandom, CorticalIdentifierParseError};
use crate::basis::cortical_area::foundation::power::{CTFoundationPower, CTFoundationPowerNMModeledStorage};


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
    use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::{NeuronLayoutOf, NeuronLayoutRestrictor};
    use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::voxel::{NeuronLayoutRestrictorVoxelBound, NeuronLayoutTagVoxel};
    use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
    use feagi_connectome::feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
    use feagi_connectome::neuron_models::models::exclusive_power::quantizations::NMExclusivePowerQuantKind;
    use feagi_connectome::neuron_models::models::kind::NMKind;
    use feagi_connectome::neuron_models::models::{NMModeled, NMNeurogenesisGenerator, NMNeurogenesisStorage, NMNeurogenesisStorageOrGenerator, NMNeurogenesisValidatedGenerator, NMNeurogenesisValidatedStorage, NMNeurogenesisValidatedStorageOrGenerator};
    use crate::basis::cortical_area::cortical_identifiers::{CorticalIdentifierInnerEncoderRandom};
    use crate::basis::cortical_area::{CorticalAllowedRegions, CorticalType};

    //region CorticalType


    /// All the neuron models supported by cortical type Foundation power
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(bound = "")]
    pub enum CTFoundationPowerNMModeled {
        ExclusivePower(NMExclusivePowerQuantKind)
    }

    impl Into<NMKind> for CTFoundationPowerNMModeled {
        fn into(self) -> NMKind {
            match self {
                CTFoundationPowerNMModeled::ExclusivePower(m) => NMKind::ExclusivePower(m),
            }
        }
    }

    /// Cortical Type that fires every burst
    pub struct CTFoundationPower(CTFoundationPowerNMModeled);

    impl CorticalType for CTFoundationPower {
        type NeuronLayout = NeuronLayoutTagVoxel;

        type NeuronLayoutRestrictor<FIQ: FeagiIndexQuantization> = NeuronLayoutRestrictorVoxelBound<
            FIQ, // Only allow 1x1x1x1
            1, 1, 1, 1,
            2, 2, 2, 2
        >;
        type InnerCorticalIDGenerator = CorticalIdentifierInnerEncoderRandom;
        const ALLOWED_REGIONS: CorticalAllowedRegions = CorticalAllowedRegions::RootOnly;

        type PossibleNeuronModelKindEnum = CTFoundationPowerNMModeled;

        type PossibleNeuronModelNeurogenesisGenerator<FIQ: FeagiIndexQuantization> = CTFoundationPowerNMModeledGenerator<FIQ>;

        fn into_neuron_model_enum(self) -> Self::PossibleNeuronModelKindEnum { self.0 }

        fn get_default_neurogenesis_generator<FIQ: FeagiIndexQuantization, NM: NeuronModel>() -> Self::PossibleNeuronModelNeurogenesisGenerator<FIQ>
        where FIQ: FeagiIndexQuantization, NM: NeuronModel
        {
            todo!()
        }
    }

    //endregion

    //region NeuronModel

    /// The NMs / Quants allowed for this Cortical Type
    pub trait CTFoundationPowerNM: NeuronModel
    {
        /// Spawns the generator precursor, effectively the default parameters for creating a cortical area
        fn spawn_neurogenesis_generator<
            FIQ: FeagiIndexQuantization,
            Res: NeuronLayoutRestrictor<FIQ, NeuronLayout=NeuronLayoutOf<FIQ, Self>>
        >() -> Self::NeurogenesisByGenerator<FIQ>;
        
        fn spawn_neurogenesis_generator_wrapped<
            FIQ: FeagiIndexQuantization,
            Res: NeuronLayoutRestrictor<FIQ, NeuronLayout=NeuronLayoutOf<FIQ, Self>>
        >() -> CTFoundationPowerNMModeledGenerator<FIQ>;
    }

    // Neuron Model: Exclusive Power
    pub mod exclusive_power {
        use serde::{Deserialize, Serialize};
        use feagi_basis::neurons::{CorticalAreaNeuronPotential, DimensionalCorticalAreaDimensions};
        use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::model_data::NeuronModelUniversalCorticalProperties;
        use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::{NeuronLayoutOf, NeuronLayoutRestrictor};
        use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
        use feagi_connectome::feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
        use feagi_connectome::neuron_models::models::exclusive_power::{NMExclusivePowerDataCorticalInternal, NMExclusivePowerDataCorticalProperties, NMExclusivePowerDataCorticalVariance, NMExclusivePowerDataNeuronInternal};
        use feagi_connectome::neuron_models::models::exclusive_power::quantizations::quant_bfloat_16::NMExclusivePowerQuantBFloat16;
        use feagi_connectome::neuron_models::models::exclusive_power::quantizations::quant_float_16::NMExclusivePowerQuantFloat16;
        use feagi_connectome::neuron_models::models::exclusive_power::quantizations::quant_float_32::NMExclusivePowerQuantFloat32;
        use feagi_connectome::neuron_models::models::exclusive_power::quantizations::quant_float_64::NMExclusivePowerQuantFloat64;
        use feagi_connectome::neuron_models::models::NMNeurogenesisGenerator;
        use crate::basis::cortical_area::foundation::power::{CTFoundationPowerNM, CTFoundationPowerNMModeledGenerator, CTFoundationPowerNMModeledValidatedGenerator};

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

            fn spawn_neurogenesis_generator_wrapped<FIQ: FeagiIndexQuantization, Res: NeuronLayoutRestrictor<FIQ, NeuronLayout=NeuronLayoutOf<FIQ, Self>>>()
                -> CTFoundationPowerNMModeledGenerator<FIQ> {
                todo!()
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

            fn spawn_neurogenesis_generator_wrapped<FIQ: FeagiIndexQuantization, Res: NeuronLayoutRestrictor<FIQ, NeuronLayout=NeuronLayoutOf<FIQ, Self>>>() -> CTFoundationPowerNMModeledGenerator<FIQ> {
                todo!()
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

            fn spawn_neurogenesis_generator_wrapped<FIQ: FeagiIndexQuantization, Res: NeuronLayoutRestrictor<FIQ, NeuronLayout=NeuronLayoutOf<FIQ, Self>>>() -> CTFoundationPowerNMModeledGenerator<FIQ> {
                todo!()
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

            fn spawn_neurogenesis_generator_wrapped<FIQ: FeagiIndexQuantization, Res: NeuronLayoutRestrictor<FIQ, NeuronLayout=NeuronLayoutOf<FIQ, Self>>>() -> CTFoundationPowerNMModeledGenerator<FIQ> {
                todo!()
            }
        }

    }

    //endregion

    //region CT Power Other Enums

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(bound = "")]
    pub enum CTFoundationPowerNMModeledGenerator<FIQ: FeagiIndexQuantization> {
        ExclusivePower(NMModeled<NMNeurogenesisStorage<FIQ>>),
    }
    
    impl<FIQ: FeagiIndexQuantization> Into<NMNeurogenesisGenerator<FIQ>> for CTFoundationPowerNMModeledGenerator<FIQ> {
        fn into(self) -> NMNeurogenesisGenerator<FIQ> {
            todo!()
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(bound = "")]
    pub enum CTFoundationPowerNMModeledValidatedGenerator<FIQ: FeagiIndexQuantization> {
        ExclusivePower(NMModeled<NMNeurogenesisValidatedGenerator<FIQ>>),
    }



    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(bound = "")]
    pub enum CTFoundationPowerNMModeledStorage<FIQ: FeagiIndexQuantization> {
        ExclusivePower(NMModeled<NMNeurogenesisStorage<FIQ>>),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(bound = "")]
    pub enum CTFoundationPowerNMModeledValidatedStorage<FIQ: FeagiIndexQuantization> {
        ExclusivePower(NMModeled<NMNeurogenesisValidatedStorage<FIQ>>),
    }



    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(bound = "")]
    pub enum CTFoundationPowerNMModeledStorageOrGenerator<FIQ: FeagiIndexQuantization> {
        ExclusivePower(NMModeled<NMNeurogenesisStorageOrGenerator<FIQ>>),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(bound = "")]
    pub enum CTFoundationPowerNMModeledValidatedStorageOrGenerator<FIQ: FeagiIndexQuantization> {
        ExclusivePower(NMModeled<NMNeurogenesisValidatedStorageOrGenerator<FIQ>>),
    }

    //endregion

}




#[derive(Copy, Clone, Debug)]
pub enum CTFoundationSubtypedKind {
    Power(CTFoundationPowerNMModeledKind),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum CTFoundationSubtypedGeneratorPrecursor<FIQ: FeagiIndexQuantization> {
    Power(CTFoundationPowerNMModeledGeneratorPrecursor<FIQ>)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum CTFoundationSubtypedStorage<FIQ: FeagiIndexQuantization> {
    Power(CTFoundationPowerNMModeledStorage<FIQ>)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum CTFoundationSubtypedStorageOrGeneratorPrecursor<FIQ: FeagiIndexQuantization> {
    Power(CTFoundationPowerNMModeledStorageOrGeneratorPrecursor<FIQ>)
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
