use core::hash::Hash;
use feagi_basis::prelude::{QuantizedUnsignedIntegerTrait, QuantizedUnsignedIntegerUnwrappedTrait};
use serde::{Deserialize, Serialize};
use feagi_basis::feagi_basis_proc_common::enum_leveled_builder;

/// The quantization level that is for structures that must be of the same quantization across
/// all burst engines in the Neural Processing Unit
pub trait FeagiIndexQuantization: Clone + Copy + Hash + PartialEq + Eq
+ core::fmt::Debug + Sync + Send + serde::Serialize + serde::de::DeserializeOwned + 'static {
    const LEVEL: FeagiIndexQuantizationLevel;

    /// Defines the quantization of the  burst index.
    type BurstIndexQuant: QuantizedUnsignedIntegerUnwrappedTrait;

    /// Cortical area indexing
    type CorticalAreaIndexCountQuant: QuantizedUnsignedIntegerUnwrappedTrait;

    /// Defines all neuron indexing (linear, voxel, etc)
    type NeuronIndexQuant: QuantizedUnsignedIntegerUnwrappedTrait;

    /// Defines the neuron bitbatch (groupings of u8, u16, or u32) indexing. Cannot be greater than `NeuronIndexQuant`!
    type NeuronBitBatchIndexQuant: QuantizedUnsignedIntegerUnwrappedTrait;

    /// Indexing of cortical mapping entries within a burst engine.
    type CorticalMappingEntryIndexCountQuant: QuantizedUnsignedIntegerUnwrappedTrait;

    /// Indexing of synapses within a burst engine
    type SynapseIndexCountQuant: QuantizedUnsignedIntegerUnwrappedTrait;

    /// Indexing of synapse aggregators, which in parallel burst engines combine multiple synapse inputs for a single neuron
    type SynapseAggregatorIndexCountQuant: QuantizedUnsignedIntegerUnwrappedTrait;

    /// Max (theoretical) number of cortical areas per burst engine
    const MAX_CORTICAL_AREA_COUNT: usize = Self::CorticalAreaIndexCountQuant::QUANT_MAX_USIZE;

    /// Max (theoretical) number of neurons per burst engine
    const MAX_NEURON_COUNT: usize = Self::NeuronIndexQuant::QUANT_MAX_USIZE;

    /// Max (theoretical) number of mapping entries per burst engine
    const MAX_CORTICAL_MAPPING_ENTRIES_COUNT: usize = Self::CorticalMappingEntryIndexCountQuant::QUANT_MAX_USIZE;

    /// Max (theoretical) number of synapses per burst engine.
    const MAX_SYNAPSES_ENTRIES_COUNT: usize = Self::SynapseIndexCountQuant::QUANT_MAX_USIZE;

    /// Max (theoretical) number of synapse aggregator slots per burst engine. Only relevant for parallel engines
    const MAX_SYNAPSE_AGGREGATOR_COUNT: usize = Self::SynapseAggregatorIndexCountQuant::QUANT_MAX_USIZE;
}

//region Quantization Levels

/// Should work with most genomes without using too much data on most desktop deployments
#[derive(Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct FeagiIndexQuantizationStandard;

impl FeagiIndexQuantization for FeagiIndexQuantizationStandard {
    const LEVEL: FeagiIndexQuantizationLevel = FeagiIndexQuantizationLevel::StandardQuantization;
    type BurstIndexQuant = u32;
    type CorticalAreaIndexCountQuant = u16;
    type NeuronIndexQuant = u32;
    type NeuronBitBatchIndexQuant = u32;
    type CorticalMappingEntryIndexCountQuant = u16;
    type SynapseIndexCountQuant = u32;
    type SynapseAggregatorIndexCountQuant = u32;
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct FeagiIndexQuantizationLow;

impl FeagiIndexQuantization for FeagiIndexQuantizationLow {
    const LEVEL: FeagiIndexQuantizationLevel = FeagiIndexQuantizationLevel::LowQuantization;
    type BurstIndexQuant = u16;
    type CorticalAreaIndexCountQuant = u16;
    type NeuronIndexQuant = u16;
    type NeuronBitBatchIndexQuant = u16;
    type CorticalMappingEntryIndexCountQuant = u16;
    type SynapseIndexCountQuant = u16;
    type SynapseAggregatorIndexCountQuant = u16;
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct FeagiIndexQuantizationMini;

impl FeagiIndexQuantization for FeagiIndexQuantizationMini {
    const LEVEL: FeagiIndexQuantizationLevel = FeagiIndexQuantizationLevel::MiniQuantization;
    type BurstIndexQuant = u16;
    type CorticalAreaIndexCountQuant = u8;
    type NeuronIndexQuant = u16;
    type NeuronBitBatchIndexQuant = u16;
    type CorticalMappingEntryIndexCountQuant = u8;
    type SynapseIndexCountQuant = u16;
    type SynapseAggregatorIndexCountQuant = u16;
}

//endregion

/// The quantization level that is for structures that must be of the same quantization across
/// all burst engines in the Neural Processing Unit as an Enum
#[repr(u8)]
#[derive(Default, Clone, Copy, Hash, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum FeagiIndexQuantizationLevel {
    /// Used throughout genome contexts. Fits most cases without taking too much memory
    #[default]
    StandardQuantization = 0,

    LowQuantization = 1,

    MiniQuantization = 2,
}

//region Nested Enums

// TODO macro to generate all the below for a given trait with zero sized impls like this,
// including generating the with_X macros and the macro for adding the family trait to the struct

enum_leveled_builder!(
    (Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize),
    FeagiIndexQuantization, FIQ, {
        (StandardQuantization, FeagiIndexQuantizationStandard)
        (LowQuantization, FeagiIndexQuantizationLow)
        (MiniQuantization, FeagiIndexQuantizationMini)
    }
);


/*
/// Maps a quantization level to a concrete type
pub trait FeagiIndexQuantizationFamily {
    type Of<FIQ: FeagiIndexQuantization>;
}

pub enum FeagiIndexQuantizationLeveled<FIQF: FeagiIndexQuantizationFamily> {
    StandardQuantization(FIQF::Of<FeagiIndexQuantizationStandard>),
    LowQuantization(FIQF::Of<FeagiIndexQuantizationLow>),
    MiniQuantization(FIQF::Of<FeagiIndexQuantizationMini>),
}


// TODO macro with_fiq!

// TODO macro to add FeagiIndexQuantizationFamily to a struct


 */


//endregion