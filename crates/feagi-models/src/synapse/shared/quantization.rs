use feagi_data::values::quantizable::DecimalQuantizationLevel;
use crate::synapse::interfacing::model_and_quantization::PackedSynapseModelTypeAndQuantization;

/// Defines the quantization used in an cortical mapping entry for synapse dynamics
/// DO NOT IMPLEMENT THIS IN ACTUAL DATA STRUCTURES! THIS IS ONLY INTENDED TO CARRY QUANTIZATION
/// CONTEXTS
pub trait SynapseModelQuantizationLevel {
    /// The number of bits dedicated to the model type
    const NUMBER_BITS_FOR_SYNAPSE_MODEL_TYPE: u8 = 5;
    /// The number of bits dedicated to the quantization level
    const NUMBER_BITS_FOR_SYNAPSE_MODEL_QUANTIZATION: u8 = 8 - Self::NUMBER_BITS_FOR_SYNAPSE_MODEL_TYPE; // 3
    const SYNAPSE_MODEL_TYPE_BITMASK: u8 = 255 << Self::NUMBER_BITS_FOR_SYNAPSE_MODEL_QUANTIZATION; // 0b1111_1000
    const SYNAPSE_MODEL_QUANTIZATION_BITMASK: u8 = 255 >> Self::NUMBER_BITS_FOR_SYNAPSE_MODEL_TYPE; // 0b0000_0111

    /// The index of the model. Make sure it does not conflict with other models
    const MODEL_INDEX: u8;

    /// Convert directly from a 'PackedSynapseModelTypeAndQuantization'. Will be safe since
    /// 'PackedSynapseModelTypeAndQuantization' is controlled
    fn from_packed_synapse_model_and_quant(packed: PackedSynapseModelTypeAndQuantization) -> Self;
}

/// Exists to ensure all synapse model quantizations share one root
pub trait SynapseModelQuantization: Clone + Copy {

    /// All quantizations used by a given neuron model quantization level. Useful for validating
    /// device compatibility. This will also be extended in extensions of this trait
    const USED_QUANTIZATION_LEVELS: &'static [DecimalQuantizationLevel]; // Don't include a default, as we will forget about it
}