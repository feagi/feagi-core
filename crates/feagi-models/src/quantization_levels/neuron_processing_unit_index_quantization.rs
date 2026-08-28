use core::hash::Hash;
use feagi_data::values::quantizable::{QuantizedUnsignedIntegerTrait, QuantizedUnsignedIntegerUnwrappedTrait};
// TODO xxhash?

/// The quantization level that is for structures that must be of the same quantization across
/// all burst engines in the Neural Processing Unit
pub trait NeuronProcessingUnitIndexQuantization: Clone + Copy + Hash + PartialEq + Eq + Sync + Send + 'static  {
    const LEVEL: NeuronProcessingUnitIndexQuantizationLevel;

    /// Defines the quantization of the NPU global burst index. This is not model configurable,
    /// rather its in sync with the global setting but also put here since some neuron models need
    /// to have this information to store "burst of last X" as a property
    type BurstIndexQuant: QuantizedUnsignedIntegerUnwrappedTrait;
}

/// The quantization level that is for structures that must be of the same quantization across
/// all burst engines in the Neural Processing Unit as an Enum
#[repr(u8)]
#[derive(Default, Debug, Clone, PartialEq)]
pub enum NeuronProcessingUnitIndexQuantizationLevel {
    /// Used throughout genome contexts. fits everything
    #[default]
    Standard32Bit = 0,

    Long64Bit = 1,

    Short16Bit = 2,
}

/// Allows for about 2 billion bursts before rolling over
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct NeuronProcessingUnitIndexQuantizationStandard32Bit;

impl NeuronProcessingUnitIndexQuantization for  NeuronProcessingUnitIndexQuantizationStandard32Bit{
    const LEVEL: NeuronProcessingUnitIndexQuantizationLevel = NeuronProcessingUnitIndexQuantizationLevel::Standard32Bit;
    type BurstIndexQuant = u32;
}

/// Allows for a lot of bursts before rolling over. This is probably excessive
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct NeuronProcessingUnitIndexQuantizationLong64Bit;

impl NeuronProcessingUnitIndexQuantization for  NeuronProcessingUnitIndexQuantizationLong64Bit{
    const LEVEL: NeuronProcessingUnitIndexQuantizationLevel = NeuronProcessingUnitIndexQuantizationLevel::Long64Bit;
    type BurstIndexQuant = u64;
}

/// Allows for about 16k bursts before rolling over. Probably not worth the memory savings
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct NeuronProcessingUnitIndexQuantizationShort16Bit;

impl NeuronProcessingUnitIndexQuantization for  NeuronProcessingUnitIndexQuantizationShort16Bit{
    const LEVEL: NeuronProcessingUnitIndexQuantizationLevel = NeuronProcessingUnitIndexQuantizationLevel::Short16Bit;
    type BurstIndexQuant = u16;
}