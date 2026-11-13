use serde::{Deserialize, Serialize};
use crate::basis::cortical_area::CorticalType;
use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use crate::basis::cortical_area::cortical_identifiers::CorticalIdentifierParseError;
use feagi_connectome::neuron_models::models::container::{NMContainer, NMContainerMarker, NMVisitor, NMVisitorMut, NMVisitorRef};
use feagi_connectome::neuron_models::models::kind::NMKind;
use crate::basis::cortical_area::{CTAccessError, CorticalTyped};


//region Cortical Subtype

/// This cortical type is for
pub trait CorticalTypeProcessing<NM>: CorticalType<NM, IdentifierInner=[u8; 6]> // uses 6 bytes as cortical ID inner
where
    NM: NeuronModel<NeuronLayoutTag=Self::NeuronLayout>
{}

/// The most common cortical type for general processing between inputs and outputs
pub mod interconnect {

    use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::{NeuronLayoutOf, NeuronLayoutRestrictor};
    use feagi_connectome::feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
    use feagi_connectome::neuron_models::models::container::{NMContainer, NMContainerMarker, NMMember, NMVisitor, NMVisitorMut, NMVisitorRef};
    use feagi_connectome::neuron_models::models::kind::NMKind;
    use crate::basis::cortical_area::{CTAccessError, CorticalTyped, CorticalTypedKind};
    use crate::basis::cortical_area::processing::CTProcessingSubtyped;
    use crate::basis::cortical_area::processing::interconnect::feagi_advanced::CTProcessingInterconnectNMFeagiAdvancedQuanted;
    use crate::basis::cortical_area::processing::interconnect::feagi_simple::CTProcessingInterconnectNMFeagiSimpleQuanted;

    /// The NMs / Quants allowed for this Cortical Type
    pub trait CTProcessingInterconnectNM: CTProcessingInterconnectMember
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
        use crate::basis::cortical_area::processing::interconnect::{CTProcessingInterconnectNM, DEFAULT_DIMENSIONS};
        use feagi_connectome::neuron_models::models::feagi_advanced::quantizations::{NMFeagiAdvancedQuantStandard32Bit};
        use feagi_connectome::neuron_models::models::container::{NMContainer, NMContainerMarker, NMMember, NMVisitor, NMVisitorMut, NMVisitorRef};
        use feagi_connectome::neuron_models::models::kind::NMKind;
        use crate::basis::cortical_area::{CTAccessError, CorticalTyped};
        use crate::basis::cortical_area::processing::CTProcessingSubtyped;
        use crate::basis::cortical_area::processing::interconnect::{CTProcessingInterconnectMember, CTProcessingInterconnectNMModeled};

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

        //region Enum Tree

        /// The data of container `C` for the FeagiAdvanced Neuron Model, at the quantizations allowed in Interconnect
        #[derive(Clone, Debug, ::serde::Serialize, ::serde::Deserialize)]
        #[serde(bound(
            serialize = "C::Of<NMFeagiAdvancedQuantStandard32Bit>: ::serde::Serialize",
            deserialize = "C::Of<NMFeagiAdvancedQuantStandard32Bit>: ::serde::Deserialize<'de>"
        ))]
        pub enum CTProcessingInterconnectNMFeagiAdvancedQuanted<C: NMContainer> {
            Standard32Bit(C::Of<NMFeagiAdvancedQuantStandard32Bit>),
        }

        impl<C: NMContainer> CTProcessingInterconnectNMFeagiAdvancedQuanted<C> {
            /// The member held, as its marker struct
            pub fn kind(&self) -> CTProcessingInterconnectNMFeagiAdvancedQuanted<NMContainerMarker> {
                match self {
                    Self::Standard32Bit(_) => CTProcessingInterconnectNMFeagiAdvancedQuanted::Standard32Bit(NMFeagiAdvancedQuantStandard32Bit),
                }
            }

            /// The neuron model and quantization held
            pub const fn nm_kind(&self) -> NMKind {
                match self {
                    Self::Standard32Bit(_) => <NMFeagiAdvancedQuantStandard32Bit as NMMember>::KIND,
                }
            }

            /// Runs the visitor on the data held, with the neuron model it is held for
            pub fn dispatch<V: NMVisitor<C>>(self, visitor: V) -> V::Output {
                match self {
                    Self::Standard32Bit(data) => visitor.visit::<NMFeagiAdvancedQuantStandard32Bit>(data),
                }
            }

            /// Runs the visitor on a reference to the data held, with the neuron model it is held for
            pub fn dispatch_ref<'a, V: NMVisitorRef<'a, C>>(&'a self, visitor: V) -> V::Output {
                match self {
                    Self::Standard32Bit(data) => visitor.visit::<NMFeagiAdvancedQuantStandard32Bit>(data),
                }
            }

            /// Runs the visitor on a mutable reference to the data held, with the neuron model it is held for
            pub fn dispatch_mut<'a, V: NMVisitorMut<'a, C>>(&'a mut self, visitor: V) -> V::Output {
                match self {
                    Self::Standard32Bit(data) => visitor.visit::<NMFeagiAdvancedQuantStandard32Bit>(data),
                }
            }
        }

        // Every leaf is a unit marker struct, so the kind is fully described by the variants taken
        impl Copy for CTProcessingInterconnectNMFeagiAdvancedQuanted<NMContainerMarker> {}

        impl PartialEq for CTProcessingInterconnectNMFeagiAdvancedQuanted<NMContainerMarker> {
            fn eq(&self, other: &Self) -> bool {
                core::mem::discriminant(self) == core::mem::discriminant(other)
            }
        }

        impl Eq for CTProcessingInterconnectNMFeagiAdvancedQuanted<NMContainerMarker> {}

        impl<C: NMContainer> From<CTProcessingInterconnectNMFeagiAdvancedQuanted<C>> for CTProcessingInterconnectNMModeled<C> {
            fn from(value: CTProcessingInterconnectNMFeagiAdvancedQuanted<C>) -> Self {
                CTProcessingInterconnectNMModeled::FeagiAdvanced(value)
            }
        }

        impl<C: NMContainer> TryFrom<CTProcessingInterconnectNMModeled<C>> for CTProcessingInterconnectNMFeagiAdvancedQuanted<C> {
            type Error = CTAccessError;

            fn try_from(value: CTProcessingInterconnectNMModeled<C>) -> Result<Self, Self::Error> {
                let actual = CorticalTyped::Processing(CTProcessingSubtyped::Interconnect(value.kind()));
                match value {
                    CTProcessingInterconnectNMModeled::FeagiAdvanced(quanted) => Ok(quanted),
                    #[allow(unreachable_patterns)]
                    _ => Err(CTAccessError::IncorrectBranch { expected: "FeagiAdvanced", actual }),
                }
            }
        }

        impl CTProcessingInterconnectMember for NMFeagiAdvancedQuantStandard32Bit {
            fn into_subtype<C: NMContainer>(data: C::Of<Self>) -> CTProcessingInterconnectNMModeled<C> {
                CTProcessingInterconnectNMModeled::FeagiAdvanced(CTProcessingInterconnectNMFeagiAdvancedQuanted::Standard32Bit(data))
            }

            fn try_from_subtype<C: NMContainer>(value: CTProcessingInterconnectNMModeled<C>) -> Result<C::Of<Self>, CTAccessError> {
                let actual = CorticalTyped::Processing(CTProcessingSubtyped::Interconnect(value.kind()));
                match value {
                    CTProcessingInterconnectNMModeled::FeagiAdvanced(CTProcessingInterconnectNMFeagiAdvancedQuanted::Standard32Bit(data)) => Ok(data),
                    #[allow(unreachable_patterns)]
                    _ => Err(CTAccessError::IncorrectMember { expected: Self::cortical_typed_kind(), actual }),
                }
            }

            fn try_ref_subtype<C: NMContainer>(value: &CTProcessingInterconnectNMModeled<C>) -> Result<&C::Of<Self>, CTAccessError> {
                match value {
                    CTProcessingInterconnectNMModeled::FeagiAdvanced(CTProcessingInterconnectNMFeagiAdvancedQuanted::Standard32Bit(data)) => Ok(data),
                    #[allow(unreachable_patterns)]
                    _ => Err(CTAccessError::IncorrectMember { expected: Self::cortical_typed_kind(), actual: CorticalTyped::Processing(CTProcessingSubtyped::Interconnect(value.kind())) }),
                }
            }

            fn try_mut_subtype<C: NMContainer>(value: &mut CTProcessingInterconnectNMModeled<C>) -> Result<&mut C::Of<Self>, CTAccessError> {
                let actual = CorticalTyped::Processing(CTProcessingSubtyped::Interconnect(value.kind()));
                match value {
                    CTProcessingInterconnectNMModeled::FeagiAdvanced(CTProcessingInterconnectNMFeagiAdvancedQuanted::Standard32Bit(data)) => Ok(data),
                    #[allow(unreachable_patterns)]
                    _ => Err(CTAccessError::IncorrectMember { expected: Self::cortical_typed_kind(), actual }),
                }
            }
        }

        //endregion
    }

    pub mod feagi_simple {
        use feagi_basis::neurons::{CorticalAreaNeuronPotential, DimensionalCorticalAreaDimensions};
        use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::model_data::NeuronModelUniversalCorticalProperties;
        use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::{NeuronLayoutOf, NeuronLayoutRestrictor};
        use feagi_connectome::feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
        use crate::basis::cortical_area::processing::interconnect::{CTProcessingInterconnectNM, DEFAULT_DIMENSIONS};
        use feagi_connectome::neuron_models::models::feagi_simple::quantizations::{NMFeagiSimpleQuantStandard32Bit};
        use feagi_connectome::neuron_models::models::container::{NMContainer, NMContainerMarker, NMMember, NMVisitor, NMVisitorMut, NMVisitorRef};
        use feagi_connectome::neuron_models::models::kind::NMKind;
        use crate::basis::cortical_area::{CTAccessError, CorticalTyped};
        use crate::basis::cortical_area::processing::CTProcessingSubtyped;
        use crate::basis::cortical_area::processing::interconnect::{CTProcessingInterconnectMember, CTProcessingInterconnectNMModeled};

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

        //region Enum Tree

        /// The data of container `C` for the FeagiSimple Neuron Model, at the quantizations allowed in Interconnect
        #[derive(Clone, Debug, ::serde::Serialize, ::serde::Deserialize)]
        #[serde(bound(
            serialize = "C::Of<NMFeagiSimpleQuantStandard32Bit>: ::serde::Serialize",
            deserialize = "C::Of<NMFeagiSimpleQuantStandard32Bit>: ::serde::Deserialize<'de>"
        ))]
        pub enum CTProcessingInterconnectNMFeagiSimpleQuanted<C: NMContainer> {
            Standard32Bit(C::Of<NMFeagiSimpleQuantStandard32Bit>),
        }

        impl<C: NMContainer> CTProcessingInterconnectNMFeagiSimpleQuanted<C> {
            /// The member held, as its marker struct
            pub fn kind(&self) -> CTProcessingInterconnectNMFeagiSimpleQuanted<NMContainerMarker> {
                match self {
                    Self::Standard32Bit(_) => CTProcessingInterconnectNMFeagiSimpleQuanted::Standard32Bit(NMFeagiSimpleQuantStandard32Bit),
                }
            }

            /// The neuron model and quantization held
            pub const fn nm_kind(&self) -> NMKind {
                match self {
                    Self::Standard32Bit(_) => <NMFeagiSimpleQuantStandard32Bit as NMMember>::KIND,
                }
            }

            /// Runs the visitor on the data held, with the neuron model it is held for
            pub fn dispatch<V: NMVisitor<C>>(self, visitor: V) -> V::Output {
                match self {
                    Self::Standard32Bit(data) => visitor.visit::<NMFeagiSimpleQuantStandard32Bit>(data),
                }
            }

            /// Runs the visitor on a reference to the data held, with the neuron model it is held for
            pub fn dispatch_ref<'a, V: NMVisitorRef<'a, C>>(&'a self, visitor: V) -> V::Output {
                match self {
                    Self::Standard32Bit(data) => visitor.visit::<NMFeagiSimpleQuantStandard32Bit>(data),
                }
            }

            /// Runs the visitor on a mutable reference to the data held, with the neuron model it is held for
            pub fn dispatch_mut<'a, V: NMVisitorMut<'a, C>>(&'a mut self, visitor: V) -> V::Output {
                match self {
                    Self::Standard32Bit(data) => visitor.visit::<NMFeagiSimpleQuantStandard32Bit>(data),
                }
            }
        }

        // Every leaf is a unit marker struct, so the kind is fully described by the variants taken
        impl Copy for CTProcessingInterconnectNMFeagiSimpleQuanted<NMContainerMarker> {}

        impl PartialEq for CTProcessingInterconnectNMFeagiSimpleQuanted<NMContainerMarker> {
            fn eq(&self, other: &Self) -> bool {
                core::mem::discriminant(self) == core::mem::discriminant(other)
            }
        }

        impl Eq for CTProcessingInterconnectNMFeagiSimpleQuanted<NMContainerMarker> {}

        impl<C: NMContainer> From<CTProcessingInterconnectNMFeagiSimpleQuanted<C>> for CTProcessingInterconnectNMModeled<C> {
            fn from(value: CTProcessingInterconnectNMFeagiSimpleQuanted<C>) -> Self {
                CTProcessingInterconnectNMModeled::FeagiSimple(value)
            }
        }

        impl<C: NMContainer> TryFrom<CTProcessingInterconnectNMModeled<C>> for CTProcessingInterconnectNMFeagiSimpleQuanted<C> {
            type Error = CTAccessError;

            fn try_from(value: CTProcessingInterconnectNMModeled<C>) -> Result<Self, Self::Error> {
                let actual = CorticalTyped::Processing(CTProcessingSubtyped::Interconnect(value.kind()));
                match value {
                    CTProcessingInterconnectNMModeled::FeagiSimple(quanted) => Ok(quanted),
                    #[allow(unreachable_patterns)]
                    _ => Err(CTAccessError::IncorrectBranch { expected: "FeagiSimple", actual }),
                }
            }
        }

        impl CTProcessingInterconnectMember for NMFeagiSimpleQuantStandard32Bit {
            fn into_subtype<C: NMContainer>(data: C::Of<Self>) -> CTProcessingInterconnectNMModeled<C> {
                CTProcessingInterconnectNMModeled::FeagiSimple(CTProcessingInterconnectNMFeagiSimpleQuanted::Standard32Bit(data))
            }

            fn try_from_subtype<C: NMContainer>(value: CTProcessingInterconnectNMModeled<C>) -> Result<C::Of<Self>, CTAccessError> {
                let actual = CorticalTyped::Processing(CTProcessingSubtyped::Interconnect(value.kind()));
                match value {
                    CTProcessingInterconnectNMModeled::FeagiSimple(CTProcessingInterconnectNMFeagiSimpleQuanted::Standard32Bit(data)) => Ok(data),
                    #[allow(unreachable_patterns)]
                    _ => Err(CTAccessError::IncorrectMember { expected: Self::cortical_typed_kind(), actual }),
                }
            }

            fn try_ref_subtype<C: NMContainer>(value: &CTProcessingInterconnectNMModeled<C>) -> Result<&C::Of<Self>, CTAccessError> {
                match value {
                    CTProcessingInterconnectNMModeled::FeagiSimple(CTProcessingInterconnectNMFeagiSimpleQuanted::Standard32Bit(data)) => Ok(data),
                    #[allow(unreachable_patterns)]
                    _ => Err(CTAccessError::IncorrectMember { expected: Self::cortical_typed_kind(), actual: CorticalTyped::Processing(CTProcessingSubtyped::Interconnect(value.kind())) }),
                }
            }

            fn try_mut_subtype<C: NMContainer>(value: &mut CTProcessingInterconnectNMModeled<C>) -> Result<&mut C::Of<Self>, CTAccessError> {
                let actual = CorticalTyped::Processing(CTProcessingSubtyped::Interconnect(value.kind()));
                match value {
                    CTProcessingInterconnectNMModeled::FeagiSimple(CTProcessingInterconnectNMFeagiSimpleQuanted::Standard32Bit(data)) => Ok(data),
                    #[allow(unreachable_patterns)]
                    _ => Err(CTAccessError::IncorrectMember { expected: Self::cortical_typed_kind(), actual }),
                }
            }
        }

        //endregion
    }

    //region Enum Tree

    /// A member (a neuron model at a quantization) allowed in Interconnect. Wraps and unwraps the data of any
    /// container straight to and from both this subtype and the whole cortical type tree
    pub trait CTProcessingInterconnectMember: NMMember {
        /// Wraps data of this member into the neuron models of this subtype
        fn into_subtype<C: NMContainer>(data: C::Of<Self>) -> CTProcessingInterconnectNMModeled<C>;

        /// Unwraps the data held, if held for this member
        fn try_from_subtype<C: NMContainer>(value: CTProcessingInterconnectNMModeled<C>) -> Result<C::Of<Self>, CTAccessError>;

        /// The data held, if held for this member
        fn try_ref_subtype<C: NMContainer>(value: &CTProcessingInterconnectNMModeled<C>) -> Result<&C::Of<Self>, CTAccessError>;

        /// The data held, if held for this member
        fn try_mut_subtype<C: NMContainer>(value: &mut CTProcessingInterconnectNMModeled<C>) -> Result<&mut C::Of<Self>, CTAccessError>;

        /// This member in this subtype, as a kind of the whole cortical type tree
        fn cortical_typed_kind() -> CorticalTypedKind {
            Self::into_cortical_typed::<NMContainerMarker>(Self::MARKER)
        }

        /// Wraps data of this member into the whole cortical type tree
        fn into_cortical_typed<C: NMContainer>(data: C::Of<Self>) -> CorticalTyped<C> {
            CorticalTyped::Processing(CTProcessingSubtyped::Interconnect(Self::into_subtype(data)))
        }

        /// Unwraps the data held, if held for this member in this subtype
        fn try_from_cortical_typed<C: NMContainer>(value: CorticalTyped<C>) -> Result<C::Of<Self>, CTAccessError> {
            let actual = value.kind();
            match value {
                CorticalTyped::Processing(CTProcessingSubtyped::Interconnect(modeled)) => Self::try_from_subtype(modeled),
                #[allow(unreachable_patterns)]
                _ => Err(CTAccessError::IncorrectMember { expected: Self::cortical_typed_kind(), actual }),
            }
        }

        /// The data held, if held for this member in this subtype
        fn try_ref_cortical_typed<C: NMContainer>(value: &CorticalTyped<C>) -> Result<&C::Of<Self>, CTAccessError> {
            match value {
                CorticalTyped::Processing(CTProcessingSubtyped::Interconnect(modeled)) => Self::try_ref_subtype(modeled),
                #[allow(unreachable_patterns)]
                _ => Err(CTAccessError::IncorrectMember { expected: Self::cortical_typed_kind(), actual: value.kind() }),
            }
        }

        /// The data held, if held for this member in this subtype
        fn try_mut_cortical_typed<C: NMContainer>(value: &mut CorticalTyped<C>) -> Result<&mut C::Of<Self>, CTAccessError> {
            let actual = value.kind();
            match value {
                CorticalTyped::Processing(CTProcessingSubtyped::Interconnect(modeled)) => Self::try_mut_subtype(modeled),
                #[allow(unreachable_patterns)]
                _ => Err(CTAccessError::IncorrectMember { expected: Self::cortical_typed_kind(), actual }),
            }
        }
    }

    /// The data of container `C` for the neuron models allowed in Interconnect, with a nested enum per model
    #[derive(Clone, Debug, ::serde::Serialize, ::serde::Deserialize)]
    #[serde(bound(
        serialize = "CTProcessingInterconnectNMFeagiAdvancedQuanted<C>: ::serde::Serialize, CTProcessingInterconnectNMFeagiSimpleQuanted<C>: ::serde::Serialize",
        deserialize = "CTProcessingInterconnectNMFeagiAdvancedQuanted<C>: ::serde::Deserialize<'de>, CTProcessingInterconnectNMFeagiSimpleQuanted<C>: ::serde::Deserialize<'de>"
    ))]
    pub enum CTProcessingInterconnectNMModeled<C: NMContainer> {
        FeagiAdvanced(CTProcessingInterconnectNMFeagiAdvancedQuanted<C>),
        FeagiSimple(CTProcessingInterconnectNMFeagiSimpleQuanted<C>),
    }

    impl<C: NMContainer> CTProcessingInterconnectNMModeled<C> {
        /// The member held, as its marker struct
        pub fn kind(&self) -> CTProcessingInterconnectNMModeled<NMContainerMarker> {
            match self {
                Self::FeagiAdvanced(quanted) => CTProcessingInterconnectNMModeled::FeagiAdvanced(quanted.kind()),
                Self::FeagiSimple(quanted) => CTProcessingInterconnectNMModeled::FeagiSimple(quanted.kind()),
            }
        }

        /// The neuron model and quantization held
        pub const fn nm_kind(&self) -> NMKind {
            match self {
                Self::FeagiAdvanced(quanted) => quanted.nm_kind(),
                Self::FeagiSimple(quanted) => quanted.nm_kind(),
            }
        }

        /// Runs the visitor on the data held, with the neuron model it is held for
        pub fn dispatch<V: NMVisitor<C>>(self, visitor: V) -> V::Output {
            match self {
                Self::FeagiAdvanced(quanted) => quanted.dispatch(visitor),
                Self::FeagiSimple(quanted) => quanted.dispatch(visitor),
            }
        }

        /// Runs the visitor on a reference to the data held, with the neuron model it is held for
        pub fn dispatch_ref<'a, V: NMVisitorRef<'a, C>>(&'a self, visitor: V) -> V::Output {
            match self {
                Self::FeagiAdvanced(quanted) => quanted.dispatch_ref(visitor),
                Self::FeagiSimple(quanted) => quanted.dispatch_ref(visitor),
            }
        }

        /// Runs the visitor on a mutable reference to the data held, with the neuron model it is held for
        pub fn dispatch_mut<'a, V: NMVisitorMut<'a, C>>(&'a mut self, visitor: V) -> V::Output {
            match self {
                Self::FeagiAdvanced(quanted) => quanted.dispatch_mut(visitor),
                Self::FeagiSimple(quanted) => quanted.dispatch_mut(visitor),
            }
        }
    }

    // Every leaf is a unit marker struct, so the kind is fully described by the variants taken
    impl Copy for CTProcessingInterconnectNMModeled<NMContainerMarker> {}

    impl PartialEq for CTProcessingInterconnectNMModeled<NMContainerMarker> {
        fn eq(&self, other: &Self) -> bool {
            match (self, other) {
                (Self::FeagiAdvanced(a), Self::FeagiAdvanced(b)) => a == b,
                (Self::FeagiSimple(a), Self::FeagiSimple(b)) => a == b,
                #[allow(unreachable_patterns)]
                _ => false,
            }
        }
    }

    impl Eq for CTProcessingInterconnectNMModeled<NMContainerMarker> {}

    impl<C: NMContainer> From<CTProcessingInterconnectNMModeled<C>> for CTProcessingSubtyped<C> {
        fn from(value: CTProcessingInterconnectNMModeled<C>) -> Self {
            CTProcessingSubtyped::Interconnect(value)
        }
    }

    impl<C: NMContainer> TryFrom<CTProcessingSubtyped<C>> for CTProcessingInterconnectNMModeled<C> {
        type Error = CTAccessError;

        fn try_from(value: CTProcessingSubtyped<C>) -> Result<Self, Self::Error> {
            let actual = CorticalTyped::Processing(value.kind());
            match value {
                CTProcessingSubtyped::Interconnect(modeled) => Ok(modeled),
                #[allow(unreachable_patterns)]
                _ => Err(CTAccessError::IncorrectBranch { expected: "Interconnect", actual }),
            }
        }
    }

    //endregion
}

//region Enum Tree

/// The data of container `C` for any subtype of the Processing cortical type, with a nested enum per subtype
#[derive(Clone, Debug, ::serde::Serialize, ::serde::Deserialize)]
#[serde(bound(
    serialize = "interconnect::CTProcessingInterconnectNMModeled<C>: ::serde::Serialize",
    deserialize = "interconnect::CTProcessingInterconnectNMModeled<C>: ::serde::Deserialize<'de>"
))]
pub enum CTProcessingSubtyped<C: NMContainer> {
    Interconnect(interconnect::CTProcessingInterconnectNMModeled<C>),
}

impl<C: NMContainer> CTProcessingSubtyped<C> {
    /// The subtype and member held, as marker structs
    pub fn kind(&self) -> CTProcessingSubtyped<NMContainerMarker> {
        match self {
            Self::Interconnect(modeled) => CTProcessingSubtyped::Interconnect(modeled.kind()),
        }
    }

    /// The neuron model and quantization held
    pub const fn nm_kind(&self) -> NMKind {
        match self {
            Self::Interconnect(modeled) => modeled.nm_kind(),
        }
    }

    /// Runs the visitor on the data held, with the neuron model it is held for
    pub fn dispatch<V: NMVisitor<C>>(self, visitor: V) -> V::Output {
        match self {
            Self::Interconnect(modeled) => modeled.dispatch(visitor),
        }
    }

    /// Runs the visitor on a reference to the data held, with the neuron model it is held for
    pub fn dispatch_ref<'a, V: NMVisitorRef<'a, C>>(&'a self, visitor: V) -> V::Output {
        match self {
            Self::Interconnect(modeled) => modeled.dispatch_ref(visitor),
        }
    }

    /// Runs the visitor on a mutable reference to the data held, with the neuron model it is held for
    pub fn dispatch_mut<'a, V: NMVisitorMut<'a, C>>(&'a mut self, visitor: V) -> V::Output {
        match self {
            Self::Interconnect(modeled) => modeled.dispatch_mut(visitor),
        }
    }
}

// Every leaf is a unit marker struct, so the kind is fully described by the variants taken
impl Copy for CTProcessingSubtyped<NMContainerMarker> {}

impl PartialEq for CTProcessingSubtyped<NMContainerMarker> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Interconnect(a), Self::Interconnect(b)) => a == b,
            #[allow(unreachable_patterns)]
            _ => false,
        }
    }
}

impl Eq for CTProcessingSubtyped<NMContainerMarker> {}

impl<C: NMContainer> From<CTProcessingSubtyped<C>> for CorticalTyped<C> {
    fn from(value: CTProcessingSubtyped<C>) -> Self {
        CorticalTyped::Processing(value)
    }
}

impl<C: NMContainer> TryFrom<CorticalTyped<C>> for CTProcessingSubtyped<C> {
    type Error = CTAccessError;

    fn try_from(value: CorticalTyped<C>) -> Result<Self, Self::Error> {
        let actual = value.kind();
        match value {
            CorticalTyped::Processing(subtyped) => Ok(subtyped),
            #[allow(unreachable_patterns)]
            _ => Err(CTAccessError::IncorrectBranch { expected: "Processing", actual }),
        }
    }
}

//endregion

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



