//! Sets the indexing of various data types, where higher quantizations can support bigger
//! collections of those items but at an increased memory cost

use crate::values::quantizable::{QuantizedUnsignedIntegerUnwrappedTrait};
use std::hash::Hash;

// TODO we may want more granular options?

/// Global Indexing across an instance of FEAGI, primarily NPU. Controlled by NPU primarily
pub trait FeagiIndexQuantization: Clone + Copy + Hash + PartialEq + Eq + Sync + 'static {
    const QUANTIZATION_LEVEL: FeagiIndexQuantizationLevel;

    /// Defines the quantization of the NPU global burst index. This is not model configurable,
    /// rather its in sync with the global setting but also put here since some neuron models need
    /// to have this information to store "burst of last X" as a property
    type GlobalBurstIndexQuant: QuantizedUnsignedIntegerUnwrappedTrait;

    /// Neuron linear indexing, linear count, voxel indexing,
    /// and voxel count quantization
    type NeuronIndexQuant: QuantizedUnsignedIntegerUnwrappedTrait;

    /// Indexing of synapses within the NPU
    type SynapseIndexCountQuant: QuantizedUnsignedIntegerUnwrappedTrait;

    /// Indexing of cortical_area areas within the NPU.
    type CorticalAreaIndexCountQuant: QuantizedUnsignedIntegerUnwrappedTrait;

    /// Indexing of cortical mapping entries within the NPU.
    type CorticalMappingEntryIndexCountQuant: QuantizedUnsignedIntegerUnwrappedTrait;
}

//region Discrete Levels

#[repr(u8)]
#[derive(Default, Debug, Clone, PartialEq)]
/// Enum that describes what `FeagiGlobalQuantization` implementation (quantization preset) to
/// follow
pub enum FeagiIndexQuantizationLevel {
    /// Used throughout genome contexts. fits everything
    #[default]
    Genomic = 0,
    // 4b Neurons, 4b Synapses, 4b FCLC Entries, 2b Burst index, 32k Cortical Areas, 32k Cortical Mapping Entries
    Standard = 1,
    // TODO tiny, mini, big, absurd
}

/// The default quantization level for most deployments. Practical balance between speed and
/// indexing size. The only level supported by some platforms
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct FeagiIndexQuantizationStandard;

impl FeagiIndexQuantization for FeagiIndexQuantizationStandard {
    const QUANTIZATION_LEVEL: FeagiIndexQuantizationLevel = FeagiIndexQuantizationLevel::Standard;
    type GlobalBurstIndexQuant = u32;
    type NeuronIndexQuant = u32;
    type SynapseIndexCountQuant = u32;
    type CorticalAreaIndexCountQuant = u16;
    type CorticalMappingEntryIndexCountQuant = u16;
}

/// Everything is 64 bit to ensure any data type can be stored, as we are not as concerned about data
/// size with genomes
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct FeagiIndexQuantizationGenomic;

impl FeagiIndexQuantization for FeagiIndexQuantizationGenomic {
    const QUANTIZATION_LEVEL: FeagiIndexQuantizationLevel = FeagiIndexQuantizationLevel::Genomic;
    type GlobalBurstIndexQuant = u64;
    type NeuronIndexQuant = u64;
    type SynapseIndexCountQuant = u64;
    type CorticalAreaIndexCountQuant = u64;
    type CorticalMappingEntryIndexCountQuant = u64;
}

//endregion
