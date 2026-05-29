use crate::quantizable_linear::base_types::QuantizedDecimalTrait;
use crate::quantizable_linear::base_types::QuantizedIndexCountTrait;

/// Defines the burst index and cortical indexing across an entire NPU / Burst engine, as it needs 
/// to be synced across neural structures
pub trait NPUGlobalQuantization {
    /// Defines the quantization of the NPU global burst index
    type GlobalBurstIndexQuant: QuantizedIndexCountTrait;
    type CorticalIndexCountQuant: QuantizedIndexCountTrait; // We want this to be global since synapses will go between cortical indexes

    /// Neuron linear indexing, linear count, voxel indexing, and voxel count quantization
    type NeuronIndexCountQuant: QuantizedIndexCountTrait; // Should be global since synapses reference these
}

/// Shared between all cortical areas within an NPU
pub trait CorticalAreasIndexQuantization {
    /// Defines the quantization of the NPU global burst index. This is not model configurable,
    /// rather its in sync with the global setting but also put here since some neuron models need
    /// to have this information to store "burst of last X" as a property
    type GlobalBurstIndexQuant: QuantizedIndexCountTrait;
    /// Also in sync with the global setting. Neuron linear indexing, linear count, voxel indexing,
    /// and voxel count quantization
    type NeuronIndexCountQuant: QuantizedIndexCountTrait;
}

/// Defines the quantization used in a cortical area for the calculation of neuron dynamics.
/// All are required to support neuron potentials, hence this is the shared base of each model's
/// implementation. Each cortical area within an NPU may have different quantization levels.
pub trait CorticalAreaModelQuantizationBase: Sized {

    /// Defines the quantization of the membrane potential of a neuron, which all models must
    /// include. This may vary between cortical areas, even of the same model
    type NeuronPotentialQuant: QuantizedDecimalTrait;
}