use crate::base_feagi_types::quantizable_types::{QuantizableUIntType, QuantizableValueType};
use crate::genomic::cortical_area::descriptors::CorticalAreaIndexQuantization;

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

// TODO keep this here?
/// Defines the burst index and cortical indexing across an entire NPU / Burst engine, as it needs 
/// to be synced across neural structures
pub trait NPUGlobalQuantization {
    /// Defines the quantization of the NPU global burst index
    type GlobalBurstIndexQuant: QuantizableUIntType;
    type CorticalIndexCountQuant: CorticalAreaIndexQuantization; // We want this to be global since synapses will go between cortical indexes

    /// Defines the per neuron and per neuron voxel index. Will have to match the one in the NPU
    type NeuronIndexVoxelCountQuant: QuantizableUIntType; // Ditto for neuron indexes
}

pub trait CorticalAreaNeuronQuantization {

    /// Defines the per neuron and per neuron voxel index. Will have to match the one in the NPU.
    /// This will have to be constant for all neurons within an NPU
    type NeuronIndexVoxelCountQuant: QuantizableUIntType;

    /// Defines the quantization of all neuron data values (namely membrane potential). This is
    /// the most likely to vary between cortical areas within a single NPU
    type NeuronValueQuant: QuantizableValueType;
}
