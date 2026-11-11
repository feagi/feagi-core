use feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::from_storage::NeurogenesisStorage;
use feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::from_storage_or_by_generator::NeurogenesisFromStorageOrByGenerator;
use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
use crate::neuron_models::models::feagi_advanced::quantizations::{NMFeagiAdvancedNeurogenesisGeneratorQuanted, NMFeagiAdvancedNeurogenesisStorageOrGeneratorQuanted, NMFeagiAdvancedNeurogenesisStorageQuanted, NMFeagiAdvancedQuantKind};
use crate::neuron_models::models::feagi_simple::quantizations::{NMFeagiSimpleNeurogenesisGeneratorQuanted, NMFeagiSimpleNeurogenesisStorageOrGeneratorQuanted, NMFeagiSimpleNeurogenesisStorageQuanted, NMFeagiSimpleQuantKind};
use crate::neuron_models::models::foundation_death::quantizations::{NMFoundationDeathNeurogenesisGeneratorQuanted, NMFoundationDeathNeurogenesisStorageOrGeneratorQuanted, NMFoundationDeathNeurogenesisStorageQuanted, NMFoundationDeathQuantKind};
use crate::neuron_models::models::foundation_power::quantizations::{NMFoundationPowerNeurogenesisGeneratorQuanted, NMFoundationPowerNeurogenesisStorageOrGeneratorQuanted, NMFoundationPowerNeurogenesisStorageQuanted, NMFoundationPowerQuantKind};
use crate::neuron_models::models::kind::NMModelKind;
use crate::neuron_models::models::memory_fire_hashing::quantizations::{NMMemoryFireHashingNeurogenesisGeneratorQuanted, NMMemoryFireHashingNeurogenesisStorageOrGeneratorQuanted, NMMemoryFireHashingNeurogenesisStorageQuanted, NMMemoryFireHashingQuantKind};

pub mod feagi_advanced;
pub mod feagi_simple;
pub mod foundation_death;
pub mod foundation_power;
pub mod memory_fire_hashing;
pub mod kind;
pub mod neurogenesis;
//region Enums



/// Narrows the Generator of a neuron model, down to the neuron model, with a nested enum
/// specifying the generator of that quant
#[derive(Clone, Debug)]
pub enum NMNeurogenesisGeneratorModeled<FIQ: FeagiIndexQuantization> {
    FeagiAdvanced(NMFeagiAdvancedNeurogenesisGeneratorQuanted<FIQ>),
    FeagiSimple(NMFeagiSimpleNeurogenesisGeneratorQuanted<FIQ>),
    FoundationDeath(NMFoundationDeathNeurogenesisGeneratorQuanted<FIQ>),
    FoundationPower(NMFoundationPowerNeurogenesisGeneratorQuanted<FIQ>),
    MemoryFireHashing(NMMemoryFireHashingNeurogenesisGeneratorQuanted<FIQ>)
}

impl<FIQ: FeagiIndexQuantization> NMNeurogenesisGeneratorModeled<FIQ> {
    /// The neuron model held, without its quantization
    pub const fn model_kind(&self) -> NMModelKind {
        match self {
            Self::FeagiAdvanced(_) => NMModelKind::FeagiAdvanced,
            Self::FeagiSimple(_) => NMModelKind::FeagiSimple,
            Self::FoundationDeath(_) => NMModelKind::FoundationDeath,
            Self::FoundationPower(_) => NMModelKind::FoundationPower,
            Self::MemoryFireHashing(_) => NMModelKind::MemoryFireHashing,
        }
    }
}

/// Narrows the Storage of a neuron model, down to the neuron model, with a nested enum
/// specifying the storage of that quant
#[derive(Clone, Debug)]
pub enum NMNeurogenesisStorageModeled<FIQ: FeagiIndexQuantization> {
    FeagiAdvanced(NMFeagiAdvancedNeurogenesisStorageQuanted<FIQ>),
    FeagiSimple(NMFeagiSimpleNeurogenesisStorageQuanted<FIQ>),
    FoundationDeath(NMFoundationDeathNeurogenesisStorageQuanted<FIQ>),
    FoundationPower(NMFoundationPowerNeurogenesisStorageQuanted<FIQ>),
    MemoryFireHashing(NMMemoryFireHashingNeurogenesisStorageQuanted<FIQ>)
}

impl<FIQ: FeagiIndexQuantization> NMNeurogenesisStorageModeled<FIQ> {
    /// The neuron model held, without its quantization
    pub const fn model_kind(&self) -> NMModelKind {
        match self {
            Self::FeagiAdvanced(_) => NMModelKind::FeagiAdvanced,
            Self::FeagiSimple(_) => NMModelKind::FeagiSimple,
            Self::FoundationDeath(_) => NMModelKind::FoundationDeath,
            Self::FoundationPower(_) => NMModelKind::FoundationPower,
            Self::MemoryFireHashing(_) => NMModelKind::MemoryFireHashing,
        }
    }
}

/// Narrows the StorageOrGenerator enum of a neuron model, down to the neuron model, with a nested enum
/// specifying the storage or generator enum of that quant
#[derive(Clone, Debug)]
pub enum NMNeurogenesisStorageOrGeneratorModeled<FIQ: FeagiIndexQuantization> {
    FeagiAdvanced(NMFeagiAdvancedNeurogenesisStorageOrGeneratorQuanted<FIQ>),
    FeagiSimple(NMFeagiSimpleNeurogenesisStorageOrGeneratorQuanted<FIQ>),
    FoundationDeath(NMFoundationDeathNeurogenesisStorageOrGeneratorQuanted<FIQ>),
    FoundationPower(NMFoundationPowerNeurogenesisStorageOrGeneratorQuanted<FIQ>),
    MemoryFireHashing(NMMemoryFireHashingNeurogenesisStorageOrGeneratorQuanted<FIQ>)
}

impl<FIQ: FeagiIndexQuantization> NMNeurogenesisStorageOrGeneratorModeled<FIQ> {
    /// The neuron model held, without its quantization
    pub const fn model_kind(&self) -> NMModelKind {
        match self {
            Self::FeagiAdvanced(_) => NMModelKind::FeagiAdvanced,
            Self::FeagiSimple(_) => NMModelKind::FeagiSimple,
            Self::FoundationDeath(_) => NMModelKind::FoundationDeath,
            Self::FoundationPower(_) => NMModelKind::FoundationPower,
            Self::MemoryFireHashing(_) => NMModelKind::MemoryFireHashing,
        }
    }
}


//endregion


// These are types that allow easy referencing to certain structs
//region Proxies

/// A struct of stored data for a neuron models data that can be used to generate that cortical area
/// as it was saved.
pub type NeurogenesisFromStorageOf<FIQ: FeagiIndexQuantization, NM: NeuronModel>
    = NeurogenesisStorage<FIQ, NM>;
/// A struct for a given neuron model that allows creating it from minimal cortical level data.
/// This can be saved in the genome. Note implementation differs by Neuron Model
pub type NeurogenesisByGeneratorTypeOf<FIQ: FeagiIndexQuantization, NM: NeuronModel>
    = NM::NeurogenesisGenerator<FIQ>;
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
    }
}