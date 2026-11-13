use core::marker::PhantomData;
use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::validation::ValidatedNeurogenesisGenerator;
use feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
#[cfg(feature = "alloc")]
use feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::from_storage::NeurogenesisByStorage;
#[cfg(feature = "alloc")]
use feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::NeurogenesisStorageOrGenerator;
#[cfg(feature = "alloc")]
use feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::validation::{ValidatedNeurogenesisStorage, ValidatedNeurogenesisStorageOrGenerator};
use crate::neuron_models::models::kind::NMKind;
use crate::neuron_models::models::feagi_advanced::quantizations::NMFeagiAdvancedQuanted;
use crate::neuron_models::models::feagi_simple::quantizations::NMFeagiSimpleQuanted;
use crate::neuron_models::models::exclusive_death::quantizations::NMExclusiveDeathQuanted;
use crate::neuron_models::models::exclusive_power::quantizations::NMExclusivePowerQuanted;
use crate::neuron_models::models::memory_fire_hashing::quantizations::NMMemoryFireHashingQuanted;

pub mod feagi_advanced;
pub mod feagi_simple;
pub mod exclusive_death;
pub mod exclusive_power;
pub mod memory_fire_hashing;
pub mod kind;


//region Containers

/// A kind of data held per neuron model and quantization, such as the generator of a cortical
/// area. The enums [`NMModeled`] and `NM*Quanted` hold the data of a container
pub trait NMContainer {
    /// The data held for neuron model `NM`
    type Of<NM: NeuronModel>;
}

/// A neuron model at a quantization, able to wrap and unwrap the data of any container into its
/// `NM*Quanted` enum
pub trait NMMember: NeuronModel<QuantKind: core::fmt::Debug> {
    /// The `NM*Quanted` enum of this neuron model
    type Quanted<C: NMContainer>: Into<NMModeled<C>> + TryFrom<NMModeled<C>>;

    fn wrap<C: NMContainer>(data: C::Of<Self>) -> Self::Quanted<C>;

    fn unwrap<C: NMContainer>(quanted: Self::Quanted<C>) -> Result<C::Of<Self>, NeuronModelEnumConversionError<Self::QuantKind>>;
}

/// The generator of a cortical area
pub struct NMNeurogenesisGenerator<FIQ>(PhantomData<FIQ>);

impl<FIQ: FeagiIndexQuantization> NMContainer for NMNeurogenesisGenerator<FIQ> {
    type Of<NM: NeuronModel> = NeurogenesisByGeneratorTypeOf<FIQ, NM>;
}

/// The stored neuron data of a cortical area
#[cfg(feature = "alloc")]
pub struct NMNeurogenesisStorage<FIQ>(PhantomData<FIQ>);

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> NMContainer for NMNeurogenesisStorage<FIQ> {
    type Of<NM: NeuronModel> = NeurogenesisByStorage<FIQ, NM>;
}

/// The stored neuron data or the generator of a cortical area
#[cfg(feature = "alloc")]
pub struct NMNeurogenesisStorageOrGenerator<FIQ>(PhantomData<FIQ>);

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> NMContainer for NMNeurogenesisStorageOrGenerator<FIQ> {
    type Of<NM: NeuronModel> = NeurogenesisStorageOrGenerator<FIQ, NM>;
}

/// The validated generator of a cortical area
pub struct NMNeurogenesisValidatedGenerator<FIQ>(PhantomData<FIQ>);

impl<FIQ: FeagiIndexQuantization> NMContainer for NMNeurogenesisValidatedGenerator<FIQ> {
    type Of<NM: NeuronModel> = ValidatedNeurogenesisGenerator<FIQ, NM>;
}

/// The validated stored neuron data of a cortical area
#[cfg(feature = "alloc")]
pub struct NMNeurogenesisValidatedStorage<FIQ>(PhantomData<FIQ>);

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> NMContainer for NMNeurogenesisValidatedStorage<FIQ> {
    type Of<NM: NeuronModel> = ValidatedNeurogenesisStorage<FIQ, NM>;
}

/// The validated stored neuron data or generator of a cortical area
#[cfg(feature = "alloc")]
pub struct NMNeurogenesisValidatedStorageOrGenerator<FIQ>(PhantomData<FIQ>);

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> NMContainer for NMNeurogenesisValidatedStorageOrGenerator<FIQ> {
    type Of<NM: NeuronModel> = ValidatedNeurogenesisStorageOrGenerator<FIQ, NM>;
}

//endregion


//region Modeled

/// Data of container `C` for any neuron model, with a nested enum specifying the quantization
pub enum NMModeled<C: NMContainer> {
    FeagiAdvanced(NMFeagiAdvancedQuanted<C>),
    FeagiSimple(NMFeagiSimpleQuanted<C>),
    ExclusiveDeath(NMExclusiveDeathQuanted<C>),
    ExclusivePower(NMExclusivePowerQuanted<C>),
    MemoryFireHashing(NMMemoryFireHashingQuanted<C>),
}

impl<C: NMContainer> NMModeled<C> {
    /// The neuron model and quantization of the data held
    pub const fn kind(&self) -> NMKind {
        match self {
            Self::FeagiAdvanced(v) => NMKind::FeagiAdvanced(v.quant_kind()),
            Self::FeagiSimple(v) => NMKind::FeagiSimple(v.quant_kind()),
            Self::ExclusiveDeath(v) => NMKind::ExclusiveDeath(v.quant_kind()),
            Self::ExclusivePower(v) => NMKind::ExclusivePower(v.quant_kind()),
            Self::MemoryFireHashing(v) => NMKind::MemoryFireHashing(v.quant_kind()),
        }
    }
}

impl<C: NMContainer> Clone for NMModeled<C>
where
    NMFeagiAdvancedQuanted<C>: Clone,
    NMFeagiSimpleQuanted<C>: Clone,
    NMExclusiveDeathQuanted<C>: Clone,
    NMExclusivePowerQuanted<C>: Clone,
    NMMemoryFireHashingQuanted<C>: Clone,
{
    fn clone(&self) -> Self {
        match self {
            Self::FeagiAdvanced(v) => Self::FeagiAdvanced(v.clone()),
            Self::FeagiSimple(v) => Self::FeagiSimple(v.clone()),
            Self::ExclusiveDeath(v) => Self::ExclusiveDeath(v.clone()),
            Self::ExclusivePower(v) => Self::ExclusivePower(v.clone()),
            Self::MemoryFireHashing(v) => Self::MemoryFireHashing(v.clone()),
        }
    }
}

impl<C: NMContainer> core::fmt::Debug for NMModeled<C>
where
    NMFeagiAdvancedQuanted<C>: core::fmt::Debug,
    NMFeagiSimpleQuanted<C>: core::fmt::Debug,
    NMExclusiveDeathQuanted<C>: core::fmt::Debug,
    NMExclusivePowerQuanted<C>: core::fmt::Debug,
    NMMemoryFireHashingQuanted<C>: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::FeagiAdvanced(v) => f.debug_tuple("FeagiAdvanced").field(v).finish(),
            Self::FeagiSimple(v) => f.debug_tuple("FeagiSimple").field(v).finish(),
            Self::ExclusiveDeath(v) => f.debug_tuple("ExclusiveDeath").field(v).finish(),
            Self::ExclusivePower(v) => f.debug_tuple("ExclusivePower").field(v).finish(),
            Self::MemoryFireHashing(v) => f.debug_tuple("MemoryFireHashing").field(v).finish(),
        }
    }
}

//endregion


// These are types that allow easy referencing to certain structs
//region Proxies

/// The generator of a neuron model, the saveable data a cortical area is generated from. Its
/// defaults are set by the cortical type
pub type NeurogenesisByGeneratorTypeOf<FIQ, NM>
    = <NM as NeuronModel>::NeurogenesisByGenerator<FIQ>;

/// Stored neuron data of a neuron model, generating the cortical area as it was saved
#[cfg(feature = "alloc")]
pub type NeurogenesisFromStorageOf<FIQ, NM>
    = NeurogenesisByStorage<FIQ, NM>;

/// Either the stored neuron data or the generator of a neuron model. Useful for exports where
/// storage is optional, to control export sizes
#[cfg(feature = "alloc")]
pub type NeurogenesisFromStorageOrByGeneratorOf<FIQ, NM>
    = NeurogenesisStorageOrGenerator<FIQ, NM>;

//endregion


/// when going from some enums to others. `K` is the kind enum of the enum being unwrapped, such
/// as the `QuantKind` of a neuron model
#[derive(Debug, thiserror::Error)]
pub enum NeuronModelEnumConversionError<K: core::fmt::Debug> {
    #[error("Unable to unwrap given enum to neuron model and / or quantization! Expected {expected:?}, but the enum holds {actual:?}")]
    IncompatibleUnwrap {
        /// The kind the enum was asked to unwrap to
        expected: K,
        /// The kind the enum actually holds
        actual: K,
    },
    #[error("Unable to unwrap given enum to neuron model! The enum holds a different neuron model")]
    IncompatibleModel,
}