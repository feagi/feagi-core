use crate::quantizable::base_types::QuantizedDecimalTrait;
use crate::quantizable::base_types::QuantizedIndexCountTrait;

/// Defines the burst index and cortical indexing across an entire NPU / Burst engine, as it needs 
/// to be synced across neural structures
pub trait NPUGlobalQuantization {
    /// Defines the quantization of the NPU global burst index
    type GlobalBurstIndexQuant: QuantizedIndexCountTrait;
    type CorticalIndexCountQuant: QuantizedIndexCountTrait; // We want this to be global since synapses will go between cortical indexes

    /// Neuron linear indexing, linear count, voxel indexing, and voxel count quantization
    type NeuronIndexCountQuant: QuantizedIndexCountTrait; // Should be global since synapses reference these
}



/// Quantization level definitions for a given neuron model. This is the base implementation,
/// specific neuron models will have their own extension of this
pub trait NeuronModelQuantizationBase {}

/// Defines the quantization used in a cortical area for a given neuron model
pub trait CorticalAreaModelQuantization<NeuronModelQuant: NeuronModelQuantizationBase > {
    /// Defines the quantization of the NPU global burst index. This is not model configurable,
    /// rather its in sync with the global setting but also put here since some neuron models need
    /// to have this information to store "burst of last X" as a property
    type GlobalBurstIndexQuant: QuantizedIndexCountTrait;
    /// Defines the quantization of the membrane potential of a neuron, which all models must
    /// include.
    type NeuronPotentialQuant: QuantizedDecimalTrait;
}