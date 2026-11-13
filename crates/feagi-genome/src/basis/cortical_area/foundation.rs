use serde::{Deserialize, Serialize};
use crate::basis::cortical_area::CorticalType;
use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::voxel::NeuronLayoutTagVoxel;
use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome::feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
use crate::basis::cortical_area::cortical_identifiers::CorticalIdentifierParseError;
use crate::basis::cortical_area::foundation::power::{CTFoundationPowerNMModeledGeneratorPrecursor, CTFoundationPowerNMModeledKind, CTFoundationPowerNMModeledStorage, CTFoundationPowerNMModeledStorageOrGeneratorPrecursor};


//region Cortical Subtype

/// These are the core cortical areas of FEAGI, that typically have some unique functionality
/// that are highly specific and possibly integrate with underlying systems. This Cortical Type
/// is unique in that pretty much every subtype has a specific single neuron model for it.
pub trait CorticalTypeFoundation<NM>: CorticalType<
    NM,
    NeuronLayout=NeuronLayoutTagVoxel, // Locked to voxel layout
    IdentifierInner=[u8; 6] // uses 6 bytes as cortical ID inner
>
where
    NM: NeuronModel<NeuronLayoutTag=NeuronLayoutTagVoxel>
{}

/// Fires every burst
pub mod power {

    //region Supported Neuron Models for Power

    use serde::{Deserialize, Serialize};
    use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::{NeuronLayoutOf, NeuronLayoutRestrictor};
    use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
    use feagi_connectome::feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
    use crate::basis::cortical_area::foundation::power::exclusive_power::{CTFoundationPowerNMExclusivePowerQuantedGeneratorPrecursor, CTFoundationPowerNMExclusivePowerQuantedKind, CTFoundationPowerNMExclusivePowerQuantedStorage, CTFoundationPowerNMExclusivePowerQuantedStorageOrGeneratorPrecursor};

    /// The NMs / Quants allowed for this Cortical Type
    pub trait CTFoundationPowerNM: NeuronModel
    {
        /// Spawns the generator precursor, effectively the default parameters for creating a cortical area
        fn spawn_neurogenesis_generator_preview<
            FIQ: FeagiIndexQuantization,
            Res: NeuronLayoutRestrictor<FIQ, NeuronLayout=NeuronLayoutOf<FIQ, Self>>
        >() -> Self::NeurogenesisGeneratorPrecursor<FIQ>;
    }

    /// Default dimensions to use for this cortical type. Power is always exactly a single neuron
    const DEFAULT_DIMENSIONS: [usize; 4] = [1, 1, 1, 1]; // TODO direct const of DimensionalCorticalAreaDimensions


    pub mod exclusive_power {
        use serde::{Deserialize, Serialize};
        use feagi_basis::neurons::{CorticalAreaNeuronPotential, DimensionalCorticalAreaDimensions};
        use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::model_data::NeuronModelUniversalCorticalProperties;
        use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::from_storage::NeurogenesisStorage;
        use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::{NeuronLayoutOf, NeuronLayoutRestrictor};
        use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
        use feagi_connectome::feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
        use feagi_connectome::neuron_models::models::exclusive_power::{NMExclusivePowerDataCorticalInternal, NMExclusivePowerDataCorticalProperties, NMExclusivePowerDataCorticalVariance, NMExclusivePowerDataNeuronInternal};
        use feagi_connectome::neuron_models::models::exclusive_power::quantizations::{NMExclusivePowerQuantedNeurogenesisGeneratorPrecursor, NMExclusivePowerQuantBFloat16, NMExclusivePowerQuantFloat16, NMExclusivePowerQuantFloat32, NMExclusivePowerQuantFloat64, NMExclusivePowerQuantedKind};
        use crate::basis::cortical_area::foundation::power::{CTFoundationPowerNM, DEFAULT_DIMENSIONS};

        impl CTFoundationPowerNM for NMExclusivePowerQuantFloat16 {
            fn spawn_neurogenesis_generator_preview<
                FIQ: FeagiIndexQuantization,
                Res: NeuronLayoutRestrictor<FIQ, NeuronLayout=NeuronLayoutOf<FIQ, Self>>>
            () -> Self::NeurogenesisGeneratorPrecursor<FIQ> {
                // Power has nothing to configure
                Self::NeurogenesisGeneratorPrecursor {
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
            fn spawn_neurogenesis_generator_preview<
                FIQ: FeagiIndexQuantization,
                Res: NeuronLayoutRestrictor<FIQ, NeuronLayout=NeuronLayoutOf<FIQ, Self>>>
            () -> Self::NeurogenesisGeneratorPrecursor<FIQ> {
                // Power has nothing to configure
                Self::NeurogenesisGeneratorPrecursor {
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
            fn spawn_neurogenesis_generator_preview<
                FIQ: FeagiIndexQuantization,
                Res: NeuronLayoutRestrictor<FIQ, NeuronLayout=NeuronLayoutOf<FIQ, Self>>>
            () -> Self::NeurogenesisGeneratorPrecursor<FIQ> {
                // Power has nothing to configure
                Self::NeurogenesisGeneratorPrecursor {
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
            fn spawn_neurogenesis_generator_preview<
                FIQ: FeagiIndexQuantization,
                Res: NeuronLayoutRestrictor<FIQ, NeuronLayout=NeuronLayoutOf<FIQ, Self>>>
            () -> Self::NeurogenesisGeneratorPrecursor<FIQ> {
                // Power has nothing to configure
                Self::NeurogenesisGeneratorPrecursor {
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

        #[derive(Copy, Clone, Debug)]
        pub enum CTFoundationPowerNMExclusivePowerQuantedKind {
            Float16(NMExclusivePowerQuantedKind),
            BFloat16(NMExclusivePowerQuantedKind),
            Float32(NMExclusivePowerQuantedKind),
            Float64(NMExclusivePowerQuantedKind),
        }

        #[derive(Debug, Clone, Serialize, Deserialize)]
        #[serde(bound = "")]
        pub enum CTFoundationPowerNMExclusivePowerQuantedGeneratorPrecursor<FIQ: FeagiIndexQuantization> {
            Float16(<NMExclusivePowerQuantFloat16 as NeuronModel>::NeurogenesisGeneratorPrecursor<FIQ>),
            BFloat16(<NMExclusivePowerQuantBFloat16 as NeuronModel>::NeurogenesisGeneratorPrecursor<FIQ>),
            Float32(<NMExclusivePowerQuantFloat32 as NeuronModel>::NeurogenesisGeneratorPrecursor<FIQ>),
            Float64(<NMExclusivePowerQuantFloat64 as NeuronModel>::NeurogenesisGeneratorPrecursor<FIQ>),
        }

        #[derive(Debug, Clone, Serialize, Deserialize)]
        #[serde(bound = "")]
        pub enum CTFoundationPowerNMExclusivePowerQuantedStorage<FIQ: FeagiIndexQuantization> {
            Float16(NeurogenesisStorage<FIQ, NMExclusivePowerQuantFloat16>),
            BFloat16(NeurogenesisStorage<FIQ, NMExclusivePowerQuantBFloat16>),
            Float32(NeurogenesisStorage<FIQ, NMExclusivePowerQuantFloat32>),
            Float64(NeurogenesisStorage<FIQ, NMExclusivePowerQuantFloat64>),
        }

        #[derive(Debug, Clone, Serialize, Deserialize)]
        #[serde(bound = "")]
        pub enum CTFoundationPowerNMExclusivePowerQuantedStorageOrGeneratorPrecursor<FIQ: FeagiIndexQuantization> {
            Float16(NMExclusivePowerQuantedNeurogenesisGeneratorPrecursor<FIQ>),
            BFloat16(NMExclusivePowerQuantedNeurogenesisGeneratorPrecursor<FIQ>),
            Float32(NMExclusivePowerQuantedNeurogenesisGeneratorPrecursor<FIQ>),
            Float64(NMExclusivePowerQuantedNeurogenesisGeneratorPrecursor<FIQ>),
        }
        

    }

    // NOTE: Power only has 1 neuron model

    #[derive(Copy, Clone, Debug)]
    pub enum CTFoundationPowerNMModeledKind {
        ExclusivePower(CTFoundationPowerNMExclusivePowerQuantedKind),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(bound = "")]
    pub enum CTFoundationPowerNMModeledGeneratorPrecursor<FIQ: FeagiIndexQuantization> {
        ExclusivePower(CTFoundationPowerNMExclusivePowerQuantedGeneratorPrecursor<FIQ>),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(bound = "")]
    pub enum CTFoundationPowerNMModeledStorage<FIQ: FeagiIndexQuantization> {
        ExclusivePower(CTFoundationPowerNMExclusivePowerQuantedStorage<FIQ>),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(bound = "")]
    pub enum CTFoundationPowerNMModeledStorageOrGeneratorPrecursor<FIQ: FeagiIndexQuantization> {
        ExclusivePower(CTFoundationPowerNMExclusivePowerQuantedStorageOrGeneratorPrecursor<FIQ>),
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
    Power([u8; 6]) = 0
}

impl CorticalIdentifierFoundation {

    /// Must be called from `CorticalIdentifier::try_from_bytes` as it assumes it called the
    /// correct inner!
    pub(crate) fn try_from_cortical_identifier_bytes(bytes: [u8; 8]) -> Result<Self, CorticalIdentifierParseError>
    {
        debug_assert_eq!(bytes[0] % 128, 0);
        Ok(
            match bytes[1] {
                0 => {
                    let [_, _, last_six @ ..] = bytes;
                    CorticalIdentifierFoundation::Power(last_six)
                },
                err => {
                    return Err(CorticalIdentifierParseError::InvalidCorticalSubtype(err))
                }
            }
        )
    }

}

//endregion
