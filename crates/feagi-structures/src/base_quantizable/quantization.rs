use crate::base_quantizable::QuantizableUIntType;
use crate::genomic::cortical_area::descriptors::CorticalAreaIndexQuantization;


/// Defines the burst index and cortical indexing across an entire NPU, as it needs to be
/// synced across structures
pub trait NPUGlobalQuantization {
    type GlobalBurstIndexQuant: QuantizableUIntType;
    type CorticalIndexCountQuant: CorticalAreaIndexQuantization; // We want this to be global since synapses will go between cortical indexes
    type NeuronIndexCountQuant: QuantizableUIntType; // Ditto for neuron indexes

}






/// Allows for communication of quantization levels at runtime
#[repr(u8)]
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum QuantizationLevel
{
    Bit8,
    Bit16,
    Bit32,
    Bit64, // NOTE: Always include this regardless of 64 bit feature flag, so we can have better error handling
}