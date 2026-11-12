use feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::from_storage::NeurogenesisStorage;
use feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::from_storage_or_by_generator::NeurogenesisFromStorageOrByGenerator;
use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
use crate::neuron_models::models::feagi_advanced::quantizations::{NMFeagiAdvancedQuantedNeurogenesisGeneratorPrecursor, NMFeagiAdvancedNeurogenesisGeneratorQuanted, NMFeagiAdvancedQuantedNeurogenesisStorageOrGenerator, NMFeagiAdvancedNeurogenesisStorageQuanted, NMFeagiAdvancedQuantKind};
use crate::neuron_models::models::feagi_simple::quantizations::{NMFeagiSimpleQuantedNeurogenesisGeneratorPrecursor, NMFeagiSimpleNeurogenesisGeneratorQuanted, NMFeagiSimpleNeurogenesisStorageOrGeneratorQuanted, NMFeagiSimpleNeurogenesisStorageQuanted, NMFeagiSimpleQuantKind};
use crate::neuron_models::models::exclusive_death::quantizations::{NMExclusiveDeathNeurogenesisGeneratorPrecursorQuanted, NMExclusiveDeathNeurogenesisGeneratorQuanted, NMExclusiveDeathNeurogenesisStorageOrGeneratorQuanted, NMExclusiveDeathNeurogenesisStorageQuanted, NMExclusiveDeathQuantKind};
use crate::neuron_models::models::exclusive_power::quantizations::{NMExclusivePowerNeurogenesisGeneratorPrecursorQuanted, NMExclusivePowerNeurogenesisGeneratorQuanted, NMExclusivePowerNeurogenesisStorageOrGeneratorQuanted, NMExclusivePowerNeurogenesisStorageQuanted, NMExclusivePowerQuantKind};
use crate::neuron_models::models::memory_fire_hashing::quantizations::{NMMemoryFireHashingNeurogenesisGeneratorPrecursorQuanted, NMMemoryFireHashingNeurogenesisGeneratorQuanted, NMMemoryFireHashingNeurogenesisStorageOrGeneratorQuanted, NMMemoryFireHashingNeurogenesisStorageQuanted, NMMemoryFireHashingQuantKind};

pub mod feagi_advanced;
pub mod feagi_simple;
pub mod exclusive_death;
pub mod exclusive_power;
pub mod memory_fire_hashing;
pub mod kind;
pub mod neurogenesis;


//region Enums

#[derive(Clone, Debug)]
pub enum NMNeurogenesisGeneratorPrecursorModeled<FIQ: FeagiIndexQuantization> {
    FeagiAdvanced(NMFeagiAdvancedQuantedNeurogenesisGeneratorPrecursor<FIQ>),
    FeagiSimple(NMFeagiSimpleQuantedNeurogenesisGeneratorPrecursor<FIQ>),
    ExclusiveDeath(NMExclusiveDeathNeurogenesisGeneratorPrecursorQuanted<FIQ>),
    ExclusivePower(NMExclusivePowerNeurogenesisGeneratorPrecursorQuanted<FIQ>),
    MemoryFireHashing(NMMemoryFireHashingNeurogenesisGeneratorPrecursorQuanted<FIQ>)
}


/// Narrows the Generator of a neuron model, down to the neuron model, with a nested enum
/// specifying the generator of that quant
#[derive(Clone, Debug)]
pub enum NMNeurogenesisGeneratorModeled<FIQ: FeagiIndexQuantization> {
    FeagiAdvanced(NMFeagiAdvancedNeurogenesisGeneratorQuanted<FIQ>),
    FeagiSimple(NMFeagiSimpleNeurogenesisGeneratorQuanted<FIQ>),
    ExclusiveDeath(NMExclusiveDeathNeurogenesisGeneratorQuanted<FIQ>),
    ExclusivePower(NMExclusivePowerNeurogenesisGeneratorQuanted<FIQ>),
    MemoryFireHashing(NMMemoryFireHashingNeurogenesisGeneratorQuanted<FIQ>)
}


/// Narrows the Storage of a neuron model, down to the neuron model, with a nested enum
/// specifying the storage of that quant
#[derive(Clone, Debug)]
pub enum NMNeurogenesisStorageModeled<FIQ: FeagiIndexQuantization> {
    FeagiAdvanced(NMFeagiAdvancedNeurogenesisStorageQuanted<FIQ>),
    FeagiSimple(NMFeagiSimpleNeurogenesisStorageQuanted<FIQ>),
    ExclusiveDeath(NMExclusiveDeathNeurogenesisStorageQuanted<FIQ>),
    ExclusivePower(NMExclusivePowerNeurogenesisStorageQuanted<FIQ>),
    MemoryFireHashing(NMMemoryFireHashingNeurogenesisStorageQuanted<FIQ>)
}

/// Narrows the StorageOrGenerator enum of a neuron model, down to the neuron model, with a nested enum
/// specifying the storage or generator enum of that quant
#[derive(Clone, Debug)]
pub enum NMNeurogenesisStorageOrGeneratorModeled<FIQ: FeagiIndexQuantization> {
    FeagiAdvanced(NMFeagiAdvancedQuantedNeurogenesisStorageOrGenerator<FIQ>),
    FeagiSimple(NMFeagiSimpleNeurogenesisStorageOrGeneratorQuanted<FIQ>),
    ExclusiveDeath(NMExclusiveDeathNeurogenesisStorageOrGeneratorQuanted<FIQ>),
    ExclusivePower(NMExclusivePowerNeurogenesisStorageOrGeneratorQuanted<FIQ>),
    MemoryFireHashing(NMMemoryFireHashingNeurogenesisStorageOrGeneratorQuanted<FIQ>)
}



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