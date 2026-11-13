use feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::from_storage::NeurogenesisStorage;
use feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::from_storage_or_by_generator::NeurogenesisFromStorageOrByGenerator;
use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
use crate::neuron_models::models::container::{NMAccessError, NMContainer, NMContainerGenerator, NMContainerGeneratorPrecursor, NMContainerStorage, NMContainerStorageOrGenerator, NMContainerStorageOrGeneratorPrecursor, NMMember, NMVisitor, NMVisitorMut, NMVisitorRef};
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
pub mod container;


//region Enums

/// The data of container `C` for any neuron model, with a nested enum per model then holding it
/// for the quantization of that model
#[derive(Clone, Debug, ::serde::Serialize, ::serde::Deserialize)]
#[serde(bound(
    serialize = "NMFeagiAdvancedQuanted<C>: ::serde::Serialize, NMFeagiSimpleQuanted<C>: ::serde::Serialize, NMExclusiveDeathQuanted<C>: ::serde::Serialize, NMExclusivePowerQuanted<C>: ::serde::Serialize, NMMemoryFireHashingQuanted<C>: ::serde::Serialize",
    deserialize = "NMFeagiAdvancedQuanted<C>: ::serde::Deserialize<'de>, NMFeagiSimpleQuanted<C>: ::serde::Deserialize<'de>, NMExclusiveDeathQuanted<C>: ::serde::Deserialize<'de>, NMExclusivePowerQuanted<C>: ::serde::Deserialize<'de>, NMMemoryFireHashingQuanted<C>: ::serde::Deserialize<'de>"
))]
pub enum NMModeled<C: NMContainer> {
    FeagiAdvanced(NMFeagiAdvancedQuanted<C>),
    FeagiSimple(NMFeagiSimpleQuanted<C>),
    ExclusiveDeath(NMExclusiveDeathQuanted<C>),
    ExclusivePower(NMExclusivePowerQuanted<C>),
    MemoryFireHashing(NMMemoryFireHashingQuanted<C>),
}

impl<C: NMContainer> NMModeled<C> {
    /// The model and quantization of the data held
    pub const fn kind(&self) -> NMKind {
        match self {
            Self::FeagiAdvanced(quanted) => quanted.kind(),
            Self::FeagiSimple(quanted) => quanted.kind(),
            Self::ExclusiveDeath(quanted) => quanted.kind(),
            Self::ExclusivePower(quanted) => quanted.kind(),
            Self::MemoryFireHashing(quanted) => quanted.kind(),
        }
    }

    /// Wraps data of member `NM`
    pub fn from_member<NMM: NMMember>(data: C::Of<NMM>) -> Self {
        NMM::into_modeled(data)
    }

    /// Unwraps the data held, if held for member `NM`
    pub fn try_into_member<NMM: NMMember>(self) -> Result<C::Of<NMM>, NMAccessError> {
        NMM::try_from_modeled(self)
    }

    /// The data held, if held for member `NM`
    pub fn try_as_member<NMM: NMMember>(&self) -> Result<&C::Of<NMM>, NMAccessError> {
        NMM::try_ref_modeled(self)
    }

    /// The data held, if held for member `NM`
    pub fn try_as_member_mut<NMM: NMMember>(&mut self) -> Result<&mut C::Of<NMM>, NMAccessError> {
        NMM::try_mut_modeled(self)
    }

    /// Runs the visitor on the data held, with the member it is held for
    pub fn dispatch<V: NMVisitor<C>>(self, visitor: V) -> V::Output {
        match self {
            Self::FeagiAdvanced(quanted) => quanted.dispatch(visitor),
            Self::FeagiSimple(quanted) => quanted.dispatch(visitor),
            Self::ExclusiveDeath(quanted) => quanted.dispatch(visitor),
            Self::ExclusivePower(quanted) => quanted.dispatch(visitor),
            Self::MemoryFireHashing(quanted) => quanted.dispatch(visitor),
        }
    }

    /// Runs the visitor on a reference to the data held, with the member it is held for
    pub fn dispatch_ref<'a, V: NMVisitorRef<'a, C>>(&'a self, visitor: V) -> V::Output {
        match self {
            Self::FeagiAdvanced(quanted) => quanted.dispatch_ref(visitor),
            Self::FeagiSimple(quanted) => quanted.dispatch_ref(visitor),
            Self::ExclusiveDeath(quanted) => quanted.dispatch_ref(visitor),
            Self::ExclusivePower(quanted) => quanted.dispatch_ref(visitor),
            Self::MemoryFireHashing(quanted) => quanted.dispatch_ref(visitor),
        }
    }

    /// Runs the visitor on a mutable reference to the data held, with the member it is held for
    pub fn dispatch_mut<'a, V: NMVisitorMut<'a, C>>(&'a mut self, visitor: V) -> V::Output {
        match self {
            Self::FeagiAdvanced(quanted) => quanted.dispatch_mut(visitor),
            Self::FeagiSimple(quanted) => quanted.dispatch_mut(visitor),
            Self::ExclusiveDeath(quanted) => quanted.dispatch_mut(visitor),
            Self::ExclusivePower(quanted) => quanted.dispatch_mut(visitor),
            Self::MemoryFireHashing(quanted) => quanted.dispatch_mut(visitor),
        }
    }
}

/// The generator precursor of any neuron model
pub type NMModeledGeneratorPrecursor<FIQ> = NMModeled<NMContainerGeneratorPrecursor<FIQ>>;

/// The generator of any neuron model
pub type NMModeledGenerator<FIQ> = NMModeled<NMContainerGenerator<FIQ>>;

/// The stored neuron data of any neuron model
pub type NMModeledStorage<FIQ> = NMModeled<NMContainerStorage<FIQ>>;

/// The stored neuron data or the generator of any neuron model
pub type NMModeledStorageOrGenerator<FIQ> = NMModeled<NMContainerStorageOrGenerator<FIQ>>;

/// The stored neuron data or the generator precursor of any neuron model
pub type NMModeledStorageOrGeneratorPrecursor<FIQ> = NMModeled<NMContainerStorageOrGeneratorPrecursor<FIQ>>;

//endregion


// These are types that allow easy referencing to certain structs
//region Proxies
/// A struct for a given neuron model that allows spawning the generator for it, as a separate
/// struct that allows for defining defaults and other implementations from the CT
pub type NeurogenesisByGeneratorPrecursorTypeOf<FIQ: FeagiIndexQuantization, NM: NeuronModel>
= NM::NeurogenesisGeneratorPrecursor<FIQ>;

/// A struct for a given neuron model that allows creating it from minimal cortical level data.
/// This can be saved in the genome. Note implementation differs by Neuron Model
pub type NeurogenesisByGeneratorTypeOf<FIQ: FeagiIndexQuantization, NM: NeuronModel>
= NM::NeurogenesisGenerator<FIQ>;

/// A struct of stored data for a neuron models data that can be used to generate that cortical area
/// as it was saved.
pub type NeurogenesisFromStorageOf<FIQ: FeagiIndexQuantization, NM: NeuronModel>
    = NeurogenesisStorage<FIQ, NM>;

/// An enum that that acts as a generator for a neuron model, that underneath can be of a storage
/// type or of the neuron models specific generator. Useful for exports where we may want
/// to optionally allow the use of storage, or not, to control export sizes
pub type NeurogenesisFromStorageOrByGeneratorOf<FIQ: FeagiIndexQuantization, NM: NeuronModel>
    = NeurogenesisFromStorageOrByGenerator<FIQ, NM, NM::NeurogenesisGenerator<FIQ>>;

//endregion
