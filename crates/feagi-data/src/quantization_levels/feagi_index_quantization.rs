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
    type NeuronIndexCountQuant: QuantizedIndexCountTrait;

    /// Indexing of synapses within the NPU
    type SynapseIndexCountQuant: QuantizedIndexCountTrait;

    /// Indexing of cortical_area areas within the NPU.
    type CorticalAreaIndexCountQuant: QuantizedIndexCountTrait;

    /// Indexing of cortical mapping entries within the NPU.
    type CorticalMappingEntryIndexCountQuant: QuantizedIndexCountTrait;
}

//region Discrete Levels

#[repr(u8)]
#[derive(Default)]
/// Enum that describes what `FeagiGlobalQuantization` implementation (quantization preset) to
/// follow
pub enum FeagiIndexQuantizationLevel {
    #[default]
    // 4b Neurons, 4b Synapses, 4b FCLC Entries, 2b Burst index, 32k Cortical Areas, 32k Cortical Mapping Entries
    Standard = 0,
    Absurd = 1,
    // TODO tiny, mini, big, absurd
}

/// The default quantization level for most deployments. Practical balance between speed and
/// indexing size. The only level supported by some platforms
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct FeagiGlobalQuantizationStandard;

impl FeagiIndexQuantization for FeagiGlobalQuantizationStandard {
    const QUANTIZATION_LEVEL: FeagiIndexQuantizationLevel = FeagiIndexQuantizationLevel::Standard;
    type GlobalBurstIndexQuant = u32;
    type NeuronIndexCountQuant = u32;
    type SynapseIndexCountQuant = u32;
    type CorticalAreaIndexCountQuant = u16;
    type CorticalMappingEntryIndexCountQuant = u16;
}

/// The largest index qunatization. everything is a u64
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct FeagiGlobalQuantizationAbsurd;

impl FeagiIndexQuantization for FeagiGlobalQuantizationAbsurd {
    const QUANTIZATION_LEVEL: FeagiIndexQuantizationLevel = FeagiIndexQuantizationLevel::Absurd;
    type GlobalBurstIndexQuant = u64;
    type NeuronIndexCountQuant = u64;
    type SynapseIndexCountQuant = u64;
    type CorticalAreaIndexCountQuant = u64;
    type CorticalMappingEntryIndexCountQuant = u64;
}

//endregion


