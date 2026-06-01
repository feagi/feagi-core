use crate::common_const_labels::NeuronModelConstLabel;
use crate::quantizable_linear::base_types::QuantizedDecimalTrait;
use crate::quantizable_linear::base_types::QuantizedIndexCountTrait;



/*
/// Defines the burst index and cortical indexing across an entire NPU / Burst engine, as it needs 
/// to be synced across neural structures
pub trait NPUGlobalQuantization {
    /// Defines the quantization of the NPU global burst index
    type GlobalBurstIndexQuant: QuantizedIndexCountTrait;
    type CorticalIndexCountQuant: QuantizedIndexCountTrait; // We want this to be global since synapses will go between cortical indexes

    /// Neuron linear indexing, linear count, voxel indexing, and voxel count quantization
    type NeuronIndexCountQuant: QuantizedIndexCountTrait; // Should be global since synapses reference these
}
 */

/// Global Indexing across an instance of FEAGI, primarily NPU. Controlled by NPU primarily
pub trait FeagiGlobalQuantization {
    /// Defines the quantization of the NPU global burst index. This is not model configurable,
    /// rather its in sync with the global setting but also put here since some neuron models need
    /// to have this information to store "burst of last X" as a property
    type GlobalBurstIndexQuant: QuantizedIndexCountTrait;

    /// Neuron linear indexing, linear count, voxel indexing,
    /// and voxel count quantization
    type NeuronIndexCountQuant: QuantizedIndexCountTrait;

    /// Indexing of synapses within the NPU
    type SynapseIndexCountQuant: QuantizedIndexCountTrait;

    /// Indexing of cortical areas within the NPU. Note that indexes are not stable outside the NPU!
    type CorticalAreaIndexCountQuant: QuantizedIndexCountTrait;

    /// Indexing of axon bundles within the NPU. Note that indexes are not stable outside the NPU!
    type AxonBundleIndexCountQuant: QuantizedIndexCountTrait;
}

/// Defines the quantization used in a cortical area for the calculation of neuron dynamics.
/// All are required to support neuron potentials, hence this is the shared base of each model's
/// implementation. Each cortical area within an NPU may have different quantization levels.
/// DO NOT IMPLEMENT THIS IN ACTUAL DATA STRUCTURES! THIS IS ONLY INTENDED TO CARRY QUANTIZATION
/// CONTEXTS
pub trait NeuronModelQuantization
{
    /// Defines the quantization of the membrane potential of a neuron, which all models must
    /// include. This may vary between cortical areas, even of the same model
    type NeuronPotentialQuant: QuantizedDecimalTrait;
}