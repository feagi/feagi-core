use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
#[cfg(feature = "alloc")]
use feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::from_storage::NeurogenesisByStorage;
#[cfg(feature = "alloc")]
use feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::NeurogenesisStorageOrGenerator;
use crate::neuron_models::models::feagi_advanced::quantizations::{NMFeagiAdvancedNeurogenesisGeneratorQuanted, NMFeagiAdvancedNeurogenesisValidatedGeneratorQuanted};
#[cfg(feature = "alloc")]
use crate::neuron_models::models::feagi_advanced::quantizations::{NMFeagiAdvancedNeurogenesisStorageQuanted, NMFeagiAdvancedNeurogenesisStorageOrGeneratorQuanted, NMFeagiAdvancedNeurogenesisValidatedStorageQuanted, NMFeagiAdvancedNeurogenesisValidatedStorageOrGeneratorQuanted};
use crate::neuron_models::models::feagi_simple::quantizations::{NMFeagiSimpleNeurogenesisGeneratorQuanted, NMFeagiSimpleNeurogenesisValidatedGeneratorQuanted};
#[cfg(feature = "alloc")]
use crate::neuron_models::models::feagi_simple::quantizations::{NMFeagiSimpleNeurogenesisStorageQuanted, NMFeagiSimpleNeurogenesisStorageOrGeneratorQuanted, NMFeagiSimpleNeurogenesisValidatedStorageQuanted, NMFeagiSimpleNeurogenesisValidatedStorageOrGeneratorQuanted};
use crate::neuron_models::models::exclusive_death::quantizations::{NMExclusiveDeathNeurogenesisGeneratorQuanted, NMExclusiveDeathNeurogenesisValidatedGeneratorQuanted};
#[cfg(feature = "alloc")]
use crate::neuron_models::models::exclusive_death::quantizations::{NMExclusiveDeathNeurogenesisStorageQuanted, NMExclusiveDeathNeurogenesisStorageOrGeneratorQuanted, NMExclusiveDeathNeurogenesisValidatedStorageQuanted, NMExclusiveDeathNeurogenesisValidatedStorageOrGeneratorQuanted};
use crate::neuron_models::models::exclusive_power::quantizations::{NMExclusivePowerNeurogenesisGeneratorQuanted, NMExclusivePowerNeurogenesisValidatedGeneratorQuanted};
#[cfg(feature = "alloc")]
use crate::neuron_models::models::exclusive_power::quantizations::{NMExclusivePowerNeurogenesisStorageQuanted, NMExclusivePowerNeurogenesisStorageOrGeneratorQuanted, NMExclusivePowerNeurogenesisValidatedStorageQuanted, NMExclusivePowerNeurogenesisValidatedStorageOrGeneratorQuanted};
use crate::neuron_models::models::memory_fire_hashing::quantizations::{NMMemoryFireHashingNeurogenesisGeneratorQuanted, NMMemoryFireHashingNeurogenesisValidatedGeneratorQuanted};
#[cfg(feature = "alloc")]
use crate::neuron_models::models::memory_fire_hashing::quantizations::{NMMemoryFireHashingNeurogenesisStorageQuanted, NMMemoryFireHashingNeurogenesisStorageOrGeneratorQuanted, NMMemoryFireHashingNeurogenesisValidatedStorageQuanted, NMMemoryFireHashingNeurogenesisValidatedStorageOrGeneratorQuanted};

pub mod feagi_advanced;
pub mod feagi_simple;
pub mod exclusive_death;
pub mod exclusive_power;
pub mod memory_fire_hashing;
pub mod kind;


//region Enums

/// Narrows the generator down to the neuron model, with a nested enum specifying the quantization
#[derive(Clone, Debug)]
pub enum NMNeurogenesisGeneratorModeled<FIQ: FeagiIndexQuantization> {
    FeagiAdvanced(NMFeagiAdvancedNeurogenesisGeneratorQuanted<FIQ>),
    FeagiSimple(NMFeagiSimpleNeurogenesisGeneratorQuanted<FIQ>),
    ExclusiveDeath(NMExclusiveDeathNeurogenesisGeneratorQuanted<FIQ>),
    ExclusivePower(NMExclusivePowerNeurogenesisGeneratorQuanted<FIQ>),
    MemoryFireHashing(NMMemoryFireHashingNeurogenesisGeneratorQuanted<FIQ>),
}

/// Narrows the stored neuron data down to the neuron model, with a nested enum specifying the quantization
#[cfg(feature = "alloc")]
#[derive(Clone, Debug)]
pub enum NMNeurogenesisStorageModeled<FIQ: FeagiIndexQuantization> {
    FeagiAdvanced(NMFeagiAdvancedNeurogenesisStorageQuanted<FIQ>),
    FeagiSimple(NMFeagiSimpleNeurogenesisStorageQuanted<FIQ>),
    ExclusiveDeath(NMExclusiveDeathNeurogenesisStorageQuanted<FIQ>),
    ExclusivePower(NMExclusivePowerNeurogenesisStorageQuanted<FIQ>),
    MemoryFireHashing(NMMemoryFireHashingNeurogenesisStorageQuanted<FIQ>),
}

/// Narrows the stored neuron data or generator down to the neuron model, with a nested enum specifying the quantization
#[cfg(feature = "alloc")]
#[derive(Clone, Debug)]
pub enum NMNeurogenesisStorageOrGeneratorModeled<FIQ: FeagiIndexQuantization> {
    FeagiAdvanced(NMFeagiAdvancedNeurogenesisStorageOrGeneratorQuanted<FIQ>),
    FeagiSimple(NMFeagiSimpleNeurogenesisStorageOrGeneratorQuanted<FIQ>),
    ExclusiveDeath(NMExclusiveDeathNeurogenesisStorageOrGeneratorQuanted<FIQ>),
    ExclusivePower(NMExclusivePowerNeurogenesisStorageOrGeneratorQuanted<FIQ>),
    MemoryFireHashing(NMMemoryFireHashingNeurogenesisStorageOrGeneratorQuanted<FIQ>),
}

/// Narrows the validated generator down to the neuron model, with a nested enum specifying the quantization
#[derive(Clone, Debug)]
pub enum NMNeurogenesisValidatedGeneratorModeled<FIQ: FeagiIndexQuantization> {
    FeagiAdvanced(NMFeagiAdvancedNeurogenesisValidatedGeneratorQuanted<FIQ>),
    FeagiSimple(NMFeagiSimpleNeurogenesisValidatedGeneratorQuanted<FIQ>),
    ExclusiveDeath(NMExclusiveDeathNeurogenesisValidatedGeneratorQuanted<FIQ>),
    ExclusivePower(NMExclusivePowerNeurogenesisValidatedGeneratorQuanted<FIQ>),
    MemoryFireHashing(NMMemoryFireHashingNeurogenesisValidatedGeneratorQuanted<FIQ>),
}

/// Narrows the validated stored neuron data down to the neuron model, with a nested enum specifying the quantization
#[cfg(feature = "alloc")]
#[derive(Clone, Debug)]
pub enum NMNeurogenesisValidatedStorageModeled<FIQ: FeagiIndexQuantization> {
    FeagiAdvanced(NMFeagiAdvancedNeurogenesisValidatedStorageQuanted<FIQ>),
    FeagiSimple(NMFeagiSimpleNeurogenesisValidatedStorageQuanted<FIQ>),
    ExclusiveDeath(NMExclusiveDeathNeurogenesisValidatedStorageQuanted<FIQ>),
    ExclusivePower(NMExclusivePowerNeurogenesisValidatedStorageQuanted<FIQ>),
    MemoryFireHashing(NMMemoryFireHashingNeurogenesisValidatedStorageQuanted<FIQ>),
}

/// Narrows the validated stored neuron data or generator down to the neuron model, with a nested enum specifying the quantization
#[cfg(feature = "alloc")]
#[derive(Clone, Debug)]
pub enum NMNeurogenesisValidatedStorageOrGeneratorModeled<FIQ: FeagiIndexQuantization> {
    FeagiAdvanced(NMFeagiAdvancedNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ>),
    FeagiSimple(NMFeagiSimpleNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ>),
    ExclusiveDeath(NMExclusiveDeathNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ>),
    ExclusivePower(NMExclusivePowerNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ>),
    MemoryFireHashing(NMMemoryFireHashingNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ>),
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