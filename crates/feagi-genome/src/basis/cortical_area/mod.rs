pub mod foundation;
pub mod processing;
pub mod cortical_identifiers;

use crate::basis::cortical_area::foundation::CTFoundationSubtyped;
use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::{CorticalLayoutContextOf, NeuronLayout, NeuronLayoutOf, NeuronLayoutError, NeuronLayoutRestrictor, NeuronLayoutTag};
use feagi_connectome::feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome::feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
use feagi_connectome::neuron_models::neurogenesis_generator_uniform_defaults::{NeurogenesisGeneratorUniformDefault};
use crate::basis::cortical_area::cortical_identifiers::CorticalAreaInnerID;
use crate::basis::cortical_area::processing::CTProcessingSubtyped;
use feagi_connectome::neuron_models::models::container::{NMContainer, NMContainerGeneratorPrecursor, NMContainerMarker, NMContainerStorage, NMContainerStorageOrGeneratorPrecursor, NMVisitor, NMVisitorMut, NMVisitorRef};
use feagi_connectome::neuron_models::models::kind::NMKind;

//region Cortical Type

/// A cortical type is a higher level description of a cortical area. It has a set layout, but
/// could be a variety of neuron layouts (of that given layout). This is first extended by another
/// trait for the specific cortical type, then implemented bya  struct for the cortical subtype
pub trait CorticalType<NM: NeuronModel<NeuronLayoutTag=Self::NeuronLayout>> {

    /// What neuron layout is this cortical type restricted to?
    type NeuronLayout: NeuronLayoutTag;

    /// What restrictions, if any, should be on the neuron layout?
    type NeuronLayoutRestrictor<FIQ: FeagiIndexQuantization>:
        NeuronLayoutRestrictor< // layouts must match
            FIQ, NeuronLayout=<Self::NeuronLayout as NeuronLayoutTag>::Layout<FIQ>
    >;

    /// What method of inner ID is used for this cortical type
    type IdentifierInner: CorticalAreaInnerID;

    /// Any additional parameters the generator needs to do
    type CorticalAreaGeneratorParameters<FIQ: FeagiIndexQuantization>;

    /// The repr(u8) enum that represents the subtype under a given type
    type CorticalSubTypeKindEnum;

    /// Each cortical subtype has a unique byte identifier (amongst that type). This is used for 
    /// cortical Identification
    const CORTICAL_AREA_SUBTYPE_KIND: Self::CorticalSubTypeKindEnum;

    /// What regions is this cortical type allowed within the genome?
    const ALLOWED_REGIONS: CorticalAllowedRegions;

    /// Get the precursor to the neurogenesis generator for this cortical type / model. This
    /// effectively is the default values for cortical generation for the given neuron model.
    /// some precursors allow changing of the parameters before submitting it for generation
    fn get_neurogenesis_generator_precursor<FIQ: FeagiIndexQuantization>()
        -> NM::NeurogenesisGeneratorPrecursor<FIQ>;

    //region Default impls



    // TODO Cortical Identifier Conversions

    //endregion
}


//region Cortical Type Enums

/// The data of container `C` for any cortical area, with a nested enum per cortical type, then
/// per subtype, then per neuron model, then per quantization. Members are reached directly through
/// the `*Member` trait of each subtype (such as `CTProcessingInterconnectMember`)
#[repr(C, u8)]
#[derive(Clone, Debug, ::serde::Serialize, ::serde::Deserialize)]
#[serde(bound(
    serialize = "CTFoundationSubtyped<C>: ::serde::Serialize, CTProcessingSubtyped<C>: ::serde::Serialize",
    deserialize = "CTFoundationSubtyped<C>: ::serde::Deserialize<'de>, CTProcessingSubtyped<C>: ::serde::Deserialize<'de>"
))]
pub enum CorticalTyped<C: NMContainer> {
    Foundation(CTFoundationSubtyped<C>) = 0,
    Processing(CTProcessingSubtyped<C>) = 1,
}

impl<C: NMContainer> CorticalTyped<C> {
    /// The cortical type, subtype and member held, as marker structs
    pub fn kind(&self) -> CorticalTypedKind {
        match self {
            Self::Foundation(subtyped) => CorticalTyped::Foundation(subtyped.kind()),
            Self::Processing(subtyped) => CorticalTyped::Processing(subtyped.kind()),
        }
    }

    /// The neuron model and quantization held
    pub const fn nm_kind(&self) -> NMKind {
        match self {
            Self::Foundation(subtyped) => subtyped.nm_kind(),
            Self::Processing(subtyped) => subtyped.nm_kind(),
        }
    }

    /// Runs the visitor on the data held, with the neuron model it is held for
    pub fn dispatch<V: NMVisitor<C>>(self, visitor: V) -> V::Output {
        match self {
            Self::Foundation(subtyped) => subtyped.dispatch(visitor),
            Self::Processing(subtyped) => subtyped.dispatch(visitor),
        }
    }

    /// Runs the visitor on a reference to the data held, with the neuron model it is held for
    pub fn dispatch_ref<'a, V: NMVisitorRef<'a, C>>(&'a self, visitor: V) -> V::Output {
        match self {
            Self::Foundation(subtyped) => subtyped.dispatch_ref(visitor),
            Self::Processing(subtyped) => subtyped.dispatch_ref(visitor),
        }
    }

    /// Runs the visitor on a mutable reference to the data held, with the neuron model it is held for
    pub fn dispatch_mut<'a, V: NMVisitorMut<'a, C>>(&'a mut self, visitor: V) -> V::Output {
        match self {
            Self::Foundation(subtyped) => subtyped.dispatch_mut(visitor),
            Self::Processing(subtyped) => subtyped.dispatch_mut(visitor),
        }
    }
}

// Every leaf is a unit marker struct, so the kind is fully described by the variants taken
impl Copy for CorticalTyped<NMContainerMarker> {}

impl PartialEq for CorticalTyped<NMContainerMarker> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Foundation(a), Self::Foundation(b)) => a == b,
            (Self::Processing(a), Self::Processing(b)) => a == b,
            #[allow(unreachable_patterns)]
            _ => false,
        }
    }
}

impl Eq for CorticalTyped<NMContainerMarker> {}

/// The type, subtype, neuron model and quantization of a cortical area
pub type CorticalTypedKind = CorticalTyped<NMContainerMarker>;

/// The generator precursor of any cortical area
pub type CorticalTypedGeneratorPrecursor<FIQ> = CorticalTyped<NMContainerGeneratorPrecursor<FIQ>>;

/// The stored neuron data of any cortical area
pub type CorticalTypedStorage<FIQ> = CorticalTyped<NMContainerStorage<FIQ>>;

/// The stored neuron data or the generator precursor of any cortical area
pub type CorticalTypedStorageOrGeneratorPrecursor<FIQ> = CorticalTyped<NMContainerStorageOrGeneratorPrecursor<FIQ>>;

/// Data of the cortical type enum tree was accessed as a part it does not hold
#[derive(Clone, Copy, PartialEq, Debug, thiserror::Error)]
pub enum CTAccessError {
    /// Accessed as a member (a neuron model at a quantization, in a cortical subtype) other than
    /// the one held
    #[error("Tried to access the cortical area member {expected:?}, but {actual:?} is held")]
    IncorrectMember { expected: CorticalTypedKind, actual: CorticalTypedKind },
    /// Accessed as a cortical type, subtype or neuron model other than the one held
    #[error("Tried to access {expected} of the cortical type tree, but {actual:?} is held")]
    IncorrectBranch { expected: &'static str, actual: CorticalTypedKind },
}

//endregion

//endregion



//region Subtype Common Traits

// TODO think of how we can specify requantization!

/// Extend any subtype with this trait to automatically add functions for using the Uniform Default
/// neuron generator with it for its neurogenesis. Only makes sense for implementations that
/// use this generator
pub trait CorticalSubTypeUsesDefaultCorticalGenerator<NM>: CorticalType<NM>
where
    NM: NeuronModel<
        NeuronLayoutTag=Self::NeuronLayout,
    >
{
    /// Gets the default values to be used in the generator. Fields can be overridden. TODO Requantization?
    fn uniform_default_generator_defaults<
        FIQ: FeagiIndexQuantization,
    >
    () -> NeurogenesisGeneratorUniformDefault<FIQ, NM>
    where
        NM: NeuronModel<NeurogenesisGenerator<FIQ>=NeurogenesisGeneratorUniformDefault<FIQ, NM>>
    ;
}


//endregion


/// Allows specifying where a cortical area of the given type can reside in the genome
#[repr(u8)]
#[derive(Copy, Clone, Debug, Hash)]
pub enum CorticalAllowedRegions {
    RootOnly,
    InnerOnly,
    RootOrInner, // look, its a bitset!
}