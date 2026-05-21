use crate::quantizable_base::decimal::QuantizedDecimalTrait;
use crate::quantizable_base::QuantizedIndexCountTrait;
use crate::quantizable_base::unsigned_integer::QuantizedUnsignedIntegerTrait;

/// Allows for communication of quantization levels at runtime
#[repr(C)]
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum QuantizationLevel
{
    Bit8 = 1,
    Bit16 = 2,
    Bit32 = 4,
    Bit64 = 8, // NOTE: Always include this regardless of 64 bit feature flag, so we can have better error handling
}

impl QuantizationLevel {
    pub fn minimum_quantization_needed_for_usize(u: usize) -> Self {
        const U8M: usize = u8::MAX as usize;
        const U16M: usize = u16::MAX as usize;
        const U32M: usize = u32::MAX as usize;

        if u < U8M {
            return QuantizationLevel::Bit8;
        }
        if u < U16M {
            return QuantizationLevel::Bit16;
        }
        if u < U32M {
            return QuantizationLevel::Bit32;
        }
        // Dont bother checking the difference between u64 and u128. If you get here you did
        // something wrong or are trying to summon more neurons than collectively exists in all
        // mankind and instantiating a such a super-intelligence is probably against Neuraville
        // TOS or something idk
        QuantizationLevel::Bit64
    }
}


/// Defines the burst index and cortical indexing across an entire NPU / Burst engine, as it needs 
/// to be synced across neural structures
pub trait NPUGlobalQuantization {
    /// Defines the quantization of the NPU global burst index
    type GlobalBurstIndexQuant: QuantizedIndexCountTrait;
    type CorticalIndexCountQuant: QuantizedIndexCountTrait; // We want this to be global since synapses will go between cortical indexes

    /// Neuron linear indexing, linear count, voxel indexing, and voxel count quantization
    type NeuronIndexCountQuant: QuantizedIndexCountTrait; // Should be global since synapses reference these
}

/// Quantizations that vary between cortical areas
pub trait CorticalAreaNeuronQuantization {
    /// Defines the quantization of all decimal (float) neuron data values (namely membrane
    /// potential).
    type NeuronDecimalQuant: QuantizedDecimalTrait;
    /// Defines the quantization of all uint neuron data values (useful for timers)
    type NeuronUintQuant: QuantizedUnsignedIntegerTrait;
}