//! Sets the indexing of various data types, where higher quantizations can support bigger
//! collections of those items but at an increased memory cost

use std::hash::Hash;
use crate::values::quantizable::QuantizedIndexCountTrait;

/// Global Indexing across an instance of FEAGI, primarily NPU. Controlled by NPU primarily
pub trait FeagiIndexQuantization: Clone + Copy + Hash + PartialEq + Eq {
    const QUANTIZATION_LEVEL: FeagiIndexQuantizationLevel;

    /// Defines the quantization of the NPU global burst index. This is not model configurable,
    /// rather its in sync with the global setting but also put here since some neuron models need
    /// to have this information to store "burst of last X" as a property
    type GlobalBurstIndexQuant: QuantizedIndexCountTrait;

    /// Neuron linear indexing, linear count, voxel indexing,
    /// and voxel count quantization
    type NeuronIndexQuant: QuantizedIndexCountTrait;

    /// Indexing of synapses within the NPU
    type SynapseIndexCountQuant: QuantizedIndexCountTrait;

    /// Indexing of cortical_area areas within the NPU.
    type CorticalAreaIndexCountQuant: QuantizedIndexCountTrait;

    /// Indexing of cortical mapping entries within the NPU.
    type CorticalMappingEntryIndexCountQuant: QuantizedIndexCountTrait;
}

//region Discrete Levels

#[repr(u8)]
#[derive(Default, Debug, Clone, PartialEq)]
/// Enum that describes what `FeagiGlobalQuantization` implementation (quantization preset) to
/// follow
pub enum FeagiIndexQuantizationLevel {
    // 4b Neurons, 4b Synapses, 4b FCLC Entries, 2b Burst index, 32k Cortical Areas, 32k Cortical Mapping Entries
    Standard = 0,
    Absurd = 1,
    /// Used throughout genome contexts. fits everything
    #[default] 
    Genomic = 2,
    // TODO tiny, mini, big, absurd
}

/// The default quantization level for most deployments. Practical balance between speed and
/// indexing size. The only level supported by some platforms
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct FeagiGlobalQuantizationStandard;

impl FeagiIndexQuantization for FeagiGlobalQuantizationStandard {
    const QUANTIZATION_LEVEL: FeagiIndexQuantizationLevel = FeagiIndexQuantizationLevel::Standard;
    type GlobalBurstIndexQuant = u32;
    type NeuronIndexQuant = u32;
    type SynapseIndexCountQuant = u32;
    type CorticalAreaIndexCountQuant = u16;
    type CorticalMappingEntryIndexCountQuant = u16;
}

/// The largest index quantization. everything is 64bit
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct FeagiGlobalQuantizationAbsurd;

impl FeagiIndexQuantization for FeagiGlobalQuantizationAbsurd {
    const QUANTIZATION_LEVEL: FeagiIndexQuantizationLevel = FeagiIndexQuantizationLevel::Absurd;
    type GlobalBurstIndexQuant = u64;
    type NeuronIndexQuant = u64;
    type SynapseIndexCountQuant = u64;
    type CorticalAreaIndexCountQuant = u64;
    type CorticalMappingEntryIndexCountQuant = u64;
}

/// Everything is 64 bit to ensure any data type can be stored, as we are not as concerned about data
/// size with genomes
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct FeagiGlobalQuantizationGenomic;

impl FeagiIndexQuantization for FeagiGlobalQuantizationGenomic {
    const QUANTIZATION_LEVEL: FeagiIndexQuantizationLevel = FeagiIndexQuantizationLevel::Genomic;
    type GlobalBurstIndexQuant = u64;
    type NeuronIndexQuant = u64;
    type SynapseIndexCountQuant = u64;
    type CorticalAreaIndexCountQuant = u64;
    type CorticalMappingEntryIndexCountQuant = u64;
}

//endregion


