use serde::{Deserialize, Serialize};
use crate::basis::cortical_area::CorticalType;
use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use crate::basis::cortical_area::cortical_identifiers::CorticalIdentifierParseError;
use feagi_connectome::neuron_models::models::container::{NMContainer, NMContainerMarker, NMVisitor, NMVisitorMut, NMVisitorRef};
use feagi_connectome::neuron_models::models::kind::NMKind;
use crate::basis::cortical_area::{CTAccessError, CorticalTyped};
use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::voxel::NeuronLayoutTagVoxel;


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

    use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::{NeuronLayoutOf, NeuronLayoutRestrictor};
    use feagi_connectome::feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
    use feagi_connectome::neuron_models::models::container::{NMContainer, NMContainerMarker, NMMember, NMVisitor, NMVisitorMut, NMVisitorRef};
    use feagi_connectome::neuron_models::models::kind::NMKind;
    use crate::basis::cortical_area::{CTAccessError, CorticalTyped, CorticalTypedKind};
    use crate::basis::cortical_area::foundation::CTFoundationSubtyped;
    use crate::basis::cortical_area::foundation::power::exclusive_power::CTFoundationPowerNMExclusivePowerQuanted;

    /// The NMs / Quants allowed for this Cortical Type
    pub trait CTFoundationPowerNM: CTFoundationPowerMember
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
        use feagi_basis::neurons::{CorticalAreaNeuronPotential, DimensionalCorticalAreaDimensions};
        use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::model_data::NeuronModelUniversalCorticalProperties;
        use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::{NeuronLayoutOf, NeuronLayoutRestrictor};
        use feagi_connectome::feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
        use feagi_connectome::neuron_models::models::exclusive_power::{NMExclusivePowerDataCorticalInternal, NMExclusivePowerDataCorticalProperties, NMExclusivePowerDataCorticalVariance, NMExclusivePowerDataNeuronInternal};
        use crate::basis::cortical_area::foundation::power::{CTFoundationPowerNM, DEFAULT_DIMENSIONS};
        use feagi_connectome::neuron_models::models::exclusive_power::quantizations::{NMExclusivePowerQuantFloat16, NMExclusivePowerQuantBFloat16, NMExclusivePowerQuantFloat32, NMExclusivePowerQuantFloat64};
        use feagi_connectome::neuron_models::models::container::{NMContainer, NMContainerMarker, NMMember, NMVisitor, NMVisitorMut, NMVisitorRef};
        use feagi_connectome::neuron_models::models::kind::NMKind;
        use crate::basis::cortical_area::{CTAccessError, CorticalTyped};
        use crate::basis::cortical_area::foundation::CTFoundationSubtyped;
        use crate::basis::cortical_area::foundation::power::{CTFoundationPowerMember, CTFoundationPowerNMModeled};

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

        //region Enum Tree

        /// The data of container `C` for the ExclusivePower Neuron Model, at the quantizations allowed in Power
        #[derive(Clone, Debug, ::serde::Serialize, ::serde::Deserialize)]
        #[serde(bound(
            serialize = "C::Of<NMExclusivePowerQuantFloat16>: ::serde::Serialize, C::Of<NMExclusivePowerQuantBFloat16>: ::serde::Serialize, C::Of<NMExclusivePowerQuantFloat32>: ::serde::Serialize, C::Of<NMExclusivePowerQuantFloat64>: ::serde::Serialize",
            deserialize = "C::Of<NMExclusivePowerQuantFloat16>: ::serde::Deserialize<'de>, C::Of<NMExclusivePowerQuantBFloat16>: ::serde::Deserialize<'de>, C::Of<NMExclusivePowerQuantFloat32>: ::serde::Deserialize<'de>, C::Of<NMExclusivePowerQuantFloat64>: ::serde::Deserialize<'de>"
        ))]
        pub enum CTFoundationPowerNMExclusivePowerQuanted<C: NMContainer> {
            Float16(C::Of<NMExclusivePowerQuantFloat16>),
            BFloat16(C::Of<NMExclusivePowerQuantBFloat16>),
            Float32(C::Of<NMExclusivePowerQuantFloat32>),
            Float64(C::Of<NMExclusivePowerQuantFloat64>),
        }

        impl<C: NMContainer> CTFoundationPowerNMExclusivePowerQuanted<C> {
            /// The member held, as its marker struct
            pub fn kind(&self) -> CTFoundationPowerNMExclusivePowerQuanted<NMContainerMarker> {
                match self {
                    Self::Float16(_) => CTFoundationPowerNMExclusivePowerQuanted::Float16(NMExclusivePowerQuantFloat16),
                    Self::BFloat16(_) => CTFoundationPowerNMExclusivePowerQuanted::BFloat16(NMExclusivePowerQuantBFloat16),
                    Self::Float32(_) => CTFoundationPowerNMExclusivePowerQuanted::Float32(NMExclusivePowerQuantFloat32),
                    Self::Float64(_) => CTFoundationPowerNMExclusivePowerQuanted::Float64(NMExclusivePowerQuantFloat64),
                }
            }

            /// The neuron model and quantization held
            pub const fn nm_kind(&self) -> NMKind {
                match self {
                    Self::Float16(_) => <NMExclusivePowerQuantFloat16 as NMMember>::KIND,
                    Self::BFloat16(_) => <NMExclusivePowerQuantBFloat16 as NMMember>::KIND,
                    Self::Float32(_) => <NMExclusivePowerQuantFloat32 as NMMember>::KIND,
                    Self::Float64(_) => <NMExclusivePowerQuantFloat64 as NMMember>::KIND,
                }
            }

            /// Runs the visitor on the data held, with the neuron model it is held for
            pub fn dispatch<V: NMVisitor<C>>(self, visitor: V) -> V::Output {
                match self {
                    Self::Float16(data) => visitor.visit::<NMExclusivePowerQuantFloat16>(data),
                    Self::BFloat16(data) => visitor.visit::<NMExclusivePowerQuantBFloat16>(data),
                    Self::Float32(data) => visitor.visit::<NMExclusivePowerQuantFloat32>(data),
                    Self::Float64(data) => visitor.visit::<NMExclusivePowerQuantFloat64>(data),
                }
            }

            /// Runs the visitor on a reference to the data held, with the neuron model it is held for
            pub fn dispatch_ref<'a, V: NMVisitorRef<'a, C>>(&'a self, visitor: V) -> V::Output {
                match self {
                    Self::Float16(data) => visitor.visit::<NMExclusivePowerQuantFloat16>(data),
                    Self::BFloat16(data) => visitor.visit::<NMExclusivePowerQuantBFloat16>(data),
                    Self::Float32(data) => visitor.visit::<NMExclusivePowerQuantFloat32>(data),
                    Self::Float64(data) => visitor.visit::<NMExclusivePowerQuantFloat64>(data),
                }
            }

            /// Runs the visitor on a mutable reference to the data held, with the neuron model it is held for
            pub fn dispatch_mut<'a, V: NMVisitorMut<'a, C>>(&'a mut self, visitor: V) -> V::Output {
                match self {
                    Self::Float16(data) => visitor.visit::<NMExclusivePowerQuantFloat16>(data),
                    Self::BFloat16(data) => visitor.visit::<NMExclusivePowerQuantBFloat16>(data),
                    Self::Float32(data) => visitor.visit::<NMExclusivePowerQuantFloat32>(data),
                    Self::Float64(data) => visitor.visit::<NMExclusivePowerQuantFloat64>(data),
                }
            }
        }

        // Every leaf is a unit marker struct, so the kind is fully described by the variants taken
        impl Copy for CTFoundationPowerNMExclusivePowerQuanted<NMContainerMarker> {}

        impl PartialEq for CTFoundationPowerNMExclusivePowerQuanted<NMContainerMarker> {
            fn eq(&self, other: &Self) -> bool {
                core::mem::discriminant(self) == core::mem::discriminant(other)
            }
        }

        impl Eq for CTFoundationPowerNMExclusivePowerQuanted<NMContainerMarker> {}

        impl<C: NMContainer> From<CTFoundationPowerNMExclusivePowerQuanted<C>> for CTFoundationPowerNMModeled<C> {
            fn from(value: CTFoundationPowerNMExclusivePowerQuanted<C>) -> Self {
                CTFoundationPowerNMModeled::ExclusivePower(value)
            }
        }

        impl<C: NMContainer> TryFrom<CTFoundationPowerNMModeled<C>> for CTFoundationPowerNMExclusivePowerQuanted<C> {
            type Error = CTAccessError;

            fn try_from(value: CTFoundationPowerNMModeled<C>) -> Result<Self, Self::Error> {
                let actual = CorticalTyped::Foundation(CTFoundationSubtyped::Power(value.kind()));
                match value {
                    CTFoundationPowerNMModeled::ExclusivePower(quanted) => Ok(quanted),
                    #[allow(unreachable_patterns)]
                    _ => Err(CTAccessError::IncorrectBranch { expected: "ExclusivePower", actual }),
                }
            }
        }

        impl CTFoundationPowerMember for NMExclusivePowerQuantFloat16 {
            fn into_subtype<C: NMContainer>(data: C::Of<Self>) -> CTFoundationPowerNMModeled<C> {
                CTFoundationPowerNMModeled::ExclusivePower(CTFoundationPowerNMExclusivePowerQuanted::Float16(data))
            }

            fn try_from_subtype<C: NMContainer>(value: CTFoundationPowerNMModeled<C>) -> Result<C::Of<Self>, CTAccessError> {
                let actual = CorticalTyped::Foundation(CTFoundationSubtyped::Power(value.kind()));
                match value {
                    CTFoundationPowerNMModeled::ExclusivePower(CTFoundationPowerNMExclusivePowerQuanted::Float16(data)) => Ok(data),
                    #[allow(unreachable_patterns)]
                    _ => Err(CTAccessError::IncorrectMember { expected: Self::cortical_typed_kind(), actual }),
                }
            }

            fn try_ref_subtype<C: NMContainer>(value: &CTFoundationPowerNMModeled<C>) -> Result<&C::Of<Self>, CTAccessError> {
                match value {
                    CTFoundationPowerNMModeled::ExclusivePower(CTFoundationPowerNMExclusivePowerQuanted::Float16(data)) => Ok(data),
                    #[allow(unreachable_patterns)]
                    _ => Err(CTAccessError::IncorrectMember { expected: Self::cortical_typed_kind(), actual: CorticalTyped::Foundation(CTFoundationSubtyped::Power(value.kind())) }),
                }
            }

            fn try_mut_subtype<C: NMContainer>(value: &mut CTFoundationPowerNMModeled<C>) -> Result<&mut C::Of<Self>, CTAccessError> {
                let actual = CorticalTyped::Foundation(CTFoundationSubtyped::Power(value.kind()));
                match value {
                    CTFoundationPowerNMModeled::ExclusivePower(CTFoundationPowerNMExclusivePowerQuanted::Float16(data)) => Ok(data),
                    #[allow(unreachable_patterns)]
                    _ => Err(CTAccessError::IncorrectMember { expected: Self::cortical_typed_kind(), actual }),
                }
            }
        }

        impl CTFoundationPowerMember for NMExclusivePowerQuantBFloat16 {
            fn into_subtype<C: NMContainer>(data: C::Of<Self>) -> CTFoundationPowerNMModeled<C> {
                CTFoundationPowerNMModeled::ExclusivePower(CTFoundationPowerNMExclusivePowerQuanted::BFloat16(data))
            }

            fn try_from_subtype<C: NMContainer>(value: CTFoundationPowerNMModeled<C>) -> Result<C::Of<Self>, CTAccessError> {
                let actual = CorticalTyped::Foundation(CTFoundationSubtyped::Power(value.kind()));
                match value {
                    CTFoundationPowerNMModeled::ExclusivePower(CTFoundationPowerNMExclusivePowerQuanted::BFloat16(data)) => Ok(data),
                    #[allow(unreachable_patterns)]
                    _ => Err(CTAccessError::IncorrectMember { expected: Self::cortical_typed_kind(), actual }),
                }
            }

            fn try_ref_subtype<C: NMContainer>(value: &CTFoundationPowerNMModeled<C>) -> Result<&C::Of<Self>, CTAccessError> {
                match value {
                    CTFoundationPowerNMModeled::ExclusivePower(CTFoundationPowerNMExclusivePowerQuanted::BFloat16(data)) => Ok(data),
                    #[allow(unreachable_patterns)]
                    _ => Err(CTAccessError::IncorrectMember { expected: Self::cortical_typed_kind(), actual: CorticalTyped::Foundation(CTFoundationSubtyped::Power(value.kind())) }),
                }
            }

            fn try_mut_subtype<C: NMContainer>(value: &mut CTFoundationPowerNMModeled<C>) -> Result<&mut C::Of<Self>, CTAccessError> {
                let actual = CorticalTyped::Foundation(CTFoundationSubtyped::Power(value.kind()));
                match value {
                    CTFoundationPowerNMModeled::ExclusivePower(CTFoundationPowerNMExclusivePowerQuanted::BFloat16(data)) => Ok(data),
                    #[allow(unreachable_patterns)]
                    _ => Err(CTAccessError::IncorrectMember { expected: Self::cortical_typed_kind(), actual }),
                }
            }
        }

        impl CTFoundationPowerMember for NMExclusivePowerQuantFloat32 {
            fn into_subtype<C: NMContainer>(data: C::Of<Self>) -> CTFoundationPowerNMModeled<C> {
                CTFoundationPowerNMModeled::ExclusivePower(CTFoundationPowerNMExclusivePowerQuanted::Float32(data))
            }

            fn try_from_subtype<C: NMContainer>(value: CTFoundationPowerNMModeled<C>) -> Result<C::Of<Self>, CTAccessError> {
                let actual = CorticalTyped::Foundation(CTFoundationSubtyped::Power(value.kind()));
                match value {
                    CTFoundationPowerNMModeled::ExclusivePower(CTFoundationPowerNMExclusivePowerQuanted::Float32(data)) => Ok(data),
                    #[allow(unreachable_patterns)]
                    _ => Err(CTAccessError::IncorrectMember { expected: Self::cortical_typed_kind(), actual }),
                }
            }

            fn try_ref_subtype<C: NMContainer>(value: &CTFoundationPowerNMModeled<C>) -> Result<&C::Of<Self>, CTAccessError> {
                match value {
                    CTFoundationPowerNMModeled::ExclusivePower(CTFoundationPowerNMExclusivePowerQuanted::Float32(data)) => Ok(data),
                    #[allow(unreachable_patterns)]
                    _ => Err(CTAccessError::IncorrectMember { expected: Self::cortical_typed_kind(), actual: CorticalTyped::Foundation(CTFoundationSubtyped::Power(value.kind())) }),
                }
            }

            fn try_mut_subtype<C: NMContainer>(value: &mut CTFoundationPowerNMModeled<C>) -> Result<&mut C::Of<Self>, CTAccessError> {
                let actual = CorticalTyped::Foundation(CTFoundationSubtyped::Power(value.kind()));
                match value {
                    CTFoundationPowerNMModeled::ExclusivePower(CTFoundationPowerNMExclusivePowerQuanted::Float32(data)) => Ok(data),
                    #[allow(unreachable_patterns)]
                    _ => Err(CTAccessError::IncorrectMember { expected: Self::cortical_typed_kind(), actual }),
                }
            }
        }

        impl CTFoundationPowerMember for NMExclusivePowerQuantFloat64 {
            fn into_subtype<C: NMContainer>(data: C::Of<Self>) -> CTFoundationPowerNMModeled<C> {
                CTFoundationPowerNMModeled::ExclusivePower(CTFoundationPowerNMExclusivePowerQuanted::Float64(data))
            }

            fn try_from_subtype<C: NMContainer>(value: CTFoundationPowerNMModeled<C>) -> Result<C::Of<Self>, CTAccessError> {
                let actual = CorticalTyped::Foundation(CTFoundationSubtyped::Power(value.kind()));
                match value {
                    CTFoundationPowerNMModeled::ExclusivePower(CTFoundationPowerNMExclusivePowerQuanted::Float64(data)) => Ok(data),
                    #[allow(unreachable_patterns)]
                    _ => Err(CTAccessError::IncorrectMember { expected: Self::cortical_typed_kind(), actual }),
                }
            }

            fn try_ref_subtype<C: NMContainer>(value: &CTFoundationPowerNMModeled<C>) -> Result<&C::Of<Self>, CTAccessError> {
                match value {
                    CTFoundationPowerNMModeled::ExclusivePower(CTFoundationPowerNMExclusivePowerQuanted::Float64(data)) => Ok(data),
                    #[allow(unreachable_patterns)]
                    _ => Err(CTAccessError::IncorrectMember { expected: Self::cortical_typed_kind(), actual: CorticalTyped::Foundation(CTFoundationSubtyped::Power(value.kind())) }),
                }
            }

            fn try_mut_subtype<C: NMContainer>(value: &mut CTFoundationPowerNMModeled<C>) -> Result<&mut C::Of<Self>, CTAccessError> {
                let actual = CorticalTyped::Foundation(CTFoundationSubtyped::Power(value.kind()));
                match value {
                    CTFoundationPowerNMModeled::ExclusivePower(CTFoundationPowerNMExclusivePowerQuanted::Float64(data)) => Ok(data),
                    #[allow(unreachable_patterns)]
                    _ => Err(CTAccessError::IncorrectMember { expected: Self::cortical_typed_kind(), actual }),
                }
            }
        }

        //endregion
    }

    //region Enum Tree

    /// A member (a neuron model at a quantization) allowed in Power. Wraps and unwraps the data of any
    /// container straight to and from both this subtype and the whole cortical type tree
    pub trait CTFoundationPowerMember: NMMember {
        /// Wraps data of this member into the neuron models of this subtype
        fn into_subtype<C: NMContainer>(data: C::Of<Self>) -> CTFoundationPowerNMModeled<C>;

        /// Unwraps the data held, if held for this member
        fn try_from_subtype<C: NMContainer>(value: CTFoundationPowerNMModeled<C>) -> Result<C::Of<Self>, CTAccessError>;

        /// The data held, if held for this member
        fn try_ref_subtype<C: NMContainer>(value: &CTFoundationPowerNMModeled<C>) -> Result<&C::Of<Self>, CTAccessError>;

        /// The data held, if held for this member
        fn try_mut_subtype<C: NMContainer>(value: &mut CTFoundationPowerNMModeled<C>) -> Result<&mut C::Of<Self>, CTAccessError>;

        /// This member in this subtype, as a kind of the whole cortical type tree
        fn cortical_typed_kind() -> CorticalTypedKind {
            Self::into_cortical_typed::<NMContainerMarker>(Self::MARKER)
        }

        /// Wraps data of this member into the whole cortical type tree
        fn into_cortical_typed<C: NMContainer>(data: C::Of<Self>) -> CorticalTyped<C> {
            CorticalTyped::Foundation(CTFoundationSubtyped::Power(Self::into_subtype(data)))
        }

        /// Unwraps the data held, if held for this member in this subtype
        fn try_from_cortical_typed<C: NMContainer>(value: CorticalTyped<C>) -> Result<C::Of<Self>, CTAccessError> {
            let actual = value.kind();
            match value {
                CorticalTyped::Foundation(CTFoundationSubtyped::Power(modeled)) => Self::try_from_subtype(modeled),
                #[allow(unreachable_patterns)]
                _ => Err(CTAccessError::IncorrectMember { expected: Self::cortical_typed_kind(), actual }),
            }
        }

        /// The data held, if held for this member in this subtype
        fn try_ref_cortical_typed<C: NMContainer>(value: &CorticalTyped<C>) -> Result<&C::Of<Self>, CTAccessError> {
            match value {
                CorticalTyped::Foundation(CTFoundationSubtyped::Power(modeled)) => Self::try_ref_subtype(modeled),
                #[allow(unreachable_patterns)]
                _ => Err(CTAccessError::IncorrectMember { expected: Self::cortical_typed_kind(), actual: value.kind() }),
            }
        }

        /// The data held, if held for this member in this subtype
        fn try_mut_cortical_typed<C: NMContainer>(value: &mut CorticalTyped<C>) -> Result<&mut C::Of<Self>, CTAccessError> {
            let actual = value.kind();
            match value {
                CorticalTyped::Foundation(CTFoundationSubtyped::Power(modeled)) => Self::try_mut_subtype(modeled),
                #[allow(unreachable_patterns)]
                _ => Err(CTAccessError::IncorrectMember { expected: Self::cortical_typed_kind(), actual }),
            }
        }
    }

    /// The data of container `C` for the neuron models allowed in Power, with a nested enum per model
    #[derive(Clone, Debug, ::serde::Serialize, ::serde::Deserialize)]
    #[serde(bound(
        serialize = "CTFoundationPowerNMExclusivePowerQuanted<C>: ::serde::Serialize",
        deserialize = "CTFoundationPowerNMExclusivePowerQuanted<C>: ::serde::Deserialize<'de>"
    ))]
    pub enum CTFoundationPowerNMModeled<C: NMContainer> {
        ExclusivePower(CTFoundationPowerNMExclusivePowerQuanted<C>),
    }

    impl<C: NMContainer> CTFoundationPowerNMModeled<C> {
        /// The member held, as its marker struct
        pub fn kind(&self) -> CTFoundationPowerNMModeled<NMContainerMarker> {
            match self {
                Self::ExclusivePower(quanted) => CTFoundationPowerNMModeled::ExclusivePower(quanted.kind()),
            }
        }

        /// The neuron model and quantization held
        pub const fn nm_kind(&self) -> NMKind {
            match self {
                Self::ExclusivePower(quanted) => quanted.nm_kind(),
            }
        }

        /// Runs the visitor on the data held, with the neuron model it is held for
        pub fn dispatch<V: NMVisitor<C>>(self, visitor: V) -> V::Output {
            match self {
                Self::ExclusivePower(quanted) => quanted.dispatch(visitor),
            }
        }

        /// Runs the visitor on a reference to the data held, with the neuron model it is held for
        pub fn dispatch_ref<'a, V: NMVisitorRef<'a, C>>(&'a self, visitor: V) -> V::Output {
            match self {
                Self::ExclusivePower(quanted) => quanted.dispatch_ref(visitor),
            }
        }

        /// Runs the visitor on a mutable reference to the data held, with the neuron model it is held for
        pub fn dispatch_mut<'a, V: NMVisitorMut<'a, C>>(&'a mut self, visitor: V) -> V::Output {
            match self {
                Self::ExclusivePower(quanted) => quanted.dispatch_mut(visitor),
            }
        }
    }

    // Every leaf is a unit marker struct, so the kind is fully described by the variants taken
    impl Copy for CTFoundationPowerNMModeled<NMContainerMarker> {}

    impl PartialEq for CTFoundationPowerNMModeled<NMContainerMarker> {
        fn eq(&self, other: &Self) -> bool {
            match (self, other) {
                (Self::ExclusivePower(a), Self::ExclusivePower(b)) => a == b,
                #[allow(unreachable_patterns)]
                _ => false,
            }
        }
    }

    impl Eq for CTFoundationPowerNMModeled<NMContainerMarker> {}

    impl<C: NMContainer> From<CTFoundationPowerNMModeled<C>> for CTFoundationSubtyped<C> {
        fn from(value: CTFoundationPowerNMModeled<C>) -> Self {
            CTFoundationSubtyped::Power(value)
        }
    }

    impl<C: NMContainer> TryFrom<CTFoundationSubtyped<C>> for CTFoundationPowerNMModeled<C> {
        type Error = CTAccessError;

        fn try_from(value: CTFoundationSubtyped<C>) -> Result<Self, Self::Error> {
            let actual = CorticalTyped::Foundation(value.kind());
            match value {
                CTFoundationSubtyped::Power(modeled) => Ok(modeled),
                #[allow(unreachable_patterns)]
                _ => Err(CTAccessError::IncorrectBranch { expected: "Power", actual }),
            }
        }
    }

    //endregion
}

//region Enum Tree

/// The data of container `C` for any subtype of the Foundation cortical type, with a nested enum per subtype
#[derive(Clone, Debug, ::serde::Serialize, ::serde::Deserialize)]
#[serde(bound(
    serialize = "power::CTFoundationPowerNMModeled<C>: ::serde::Serialize",
    deserialize = "power::CTFoundationPowerNMModeled<C>: ::serde::Deserialize<'de>"
))]
pub enum CTFoundationSubtyped<C: NMContainer> {
    Power(power::CTFoundationPowerNMModeled<C>),
}

impl<C: NMContainer> CTFoundationSubtyped<C> {
    /// The subtype and member held, as marker structs
    pub fn kind(&self) -> CTFoundationSubtyped<NMContainerMarker> {
        match self {
            Self::Power(modeled) => CTFoundationSubtyped::Power(modeled.kind()),
        }
    }

    /// The neuron model and quantization held
    pub const fn nm_kind(&self) -> NMKind {
        match self {
            Self::Power(modeled) => modeled.nm_kind(),
        }
    }

    /// Runs the visitor on the data held, with the neuron model it is held for
    pub fn dispatch<V: NMVisitor<C>>(self, visitor: V) -> V::Output {
        match self {
            Self::Power(modeled) => modeled.dispatch(visitor),
        }
    }

    /// Runs the visitor on a reference to the data held, with the neuron model it is held for
    pub fn dispatch_ref<'a, V: NMVisitorRef<'a, C>>(&'a self, visitor: V) -> V::Output {
        match self {
            Self::Power(modeled) => modeled.dispatch_ref(visitor),
        }
    }

    /// Runs the visitor on a mutable reference to the data held, with the neuron model it is held for
    pub fn dispatch_mut<'a, V: NMVisitorMut<'a, C>>(&'a mut self, visitor: V) -> V::Output {
        match self {
            Self::Power(modeled) => modeled.dispatch_mut(visitor),
        }
    }
}

// Every leaf is a unit marker struct, so the kind is fully described by the variants taken
impl Copy for CTFoundationSubtyped<NMContainerMarker> {}

impl PartialEq for CTFoundationSubtyped<NMContainerMarker> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Power(a), Self::Power(b)) => a == b,
            #[allow(unreachable_patterns)]
            _ => false,
        }
    }
}

impl Eq for CTFoundationSubtyped<NMContainerMarker> {}

impl<C: NMContainer> From<CTFoundationSubtyped<C>> for CorticalTyped<C> {
    fn from(value: CTFoundationSubtyped<C>) -> Self {
        CorticalTyped::Foundation(value)
    }
}

impl<C: NMContainer> TryFrom<CorticalTyped<C>> for CTFoundationSubtyped<C> {
    type Error = CTAccessError;

    fn try_from(value: CorticalTyped<C>) -> Result<Self, Self::Error> {
        let actual = value.kind();
        match value {
            CorticalTyped::Foundation(subtyped) => Ok(subtyped),
            #[allow(unreachable_patterns)]
            _ => Err(CTAccessError::IncorrectBranch { expected: "Foundation", actual }),
        }
    }
}

//endregion

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
