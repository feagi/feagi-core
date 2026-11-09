//! The `NeuronModel` model family, of every neuron model at each of its quantization levels. The
//! items written by hand are here, the kinds, enums, member implementations, and dispatch macros are
//! generated from the template below. Every item is reached publicly through
//! `connectome_basis::neuron_model`

pub(super) use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome_proc::{define_model_family_template, expand_model_family};

/// A kind of data held for any member of the `NeuronModel` family, such as `NMEnum<C>` holds. Meant
/// to be implemented on a zero sized struct per kind of data
pub trait NMContainer {
    /// The data held for member `M` of the family
    type Of<M: NeuronModel>;
}

/// A member of the `NeuronModel` family, a quantization level of one of its models. Converts data
/// of any container of the family between itself and `NMEnum`. Implemented for every marker struct
/// by the family template
pub trait NMMember: NeuronModel + Sized {
    /// The model this is a member of, as a whole across its quantization levels. A type distinct
    /// per model, whose layout is the layout of the model
    type Tag: NeuronModel;

    /// The member of the family this is
    const KIND: NMKind;

    /// Wraps data of this member
    fn into_family_enum<C: NMContainer>(data: C::Of<Self>) -> NMEnum<C>;

    /// Unwraps the data held, if held for this member
    fn try_from_family_enum<C: NMContainer>(value: NMEnum<C>) -> Result<C::Of<Self>, NMAccessError>;

    /// The data held, if held for this member
    fn try_ref_family_enum<C: NMContainer>(value: &NMEnum<C>) -> Result<&C::Of<Self>, NMAccessError>;

    /// The data held, if held for this member
    fn try_mut_family_enum<C: NMContainer>(value: &mut NMEnum<C>) -> Result<&mut C::Of<Self>, NMAccessError>;
}

/// Data of the `NeuronModel` family was accessed as a member or model it does not hold
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NMAccessError {
    /// Accessed as a member of the family other than the one held
    IncorrectMember { expected: NMKind, actual: NMKind },
    /// Accessed as a model of the family other than the one held
    IncorrectModel { expected: &'static str, actual: NMKind },
}

impl core::fmt::Display for NMAccessError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::IncorrectMember { expected, actual } => write!(f, "Tried to access the NeuronModel `{}` when it was actually `{}`", expected, actual),
            Self::IncorrectModel { expected, actual } => write!(f, "Tried to access a NeuronModel of model `{}` when it was actually `{}`", expected, actual),
        }
    }
}

impl core::error::Error for NMAccessError {}

define_model_family_template! {
    /// Every neuron model, each at any of its quantization levels
    Family: NeuronModel,

    Models: {
        /// Always fires
        FoundationPower: {
            Float16,
            BFloat16,
            Float32,
            Float64,
        }

        /// Kills the brain
        FoundationDeath: {
            Float16,
            BFloat16,
            Float32,
            Float64,
        }

        /// Feagi Variant of Leaky integrate and fire, suitable for most needs
        FeagiSimple: {
            DefaultQuant,
        }

        /// Feagi Variant of Leaky integrate and fire, with additional advanced parameters at a computational and memory cost
        FeagiAdvanced: {
            DefaultQuant,
        }

        /// Gains Memories by the hash of incoming fire patterns
        MemoryFireHashing: {
            DefaultQuant,
        }

    }
}

neuron_model_family_template!(expand_model_family);
