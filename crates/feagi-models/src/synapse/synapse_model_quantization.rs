use std::hash::Hash;
use feagi_data::values::quantizable::{DecimalQuantizationLevel, QuantizedDecimalTrait};
use crate::synapse::model_generated::model_type_and_quantization::{SynapseModelTypeAndQuantizationNested, SynapseModelTypeAndQuantizationPacked, SynapseModelType};

/// Common root trait shared by all Synapse Model Quantizations. This trait should be extended
/// by the given synapse model to add any quantization parameters for their given data
pub trait SynapseModelQuantization: Clone + Default {

    /// Defines the quantization incoming and outgoing signals will be quantized to
    type JunctionPotentialQuant: QuantizedDecimalTrait;

    /// A flat enum value denoting what type of synapse model this synapse model instance is
    const SYNAPSE_MODEL: SynapseModelType;

    /// The type of enum that can denote the quantization level of this synapse model
    type QuantLevelType: SynapseModelQuantizationLevel;

    /// A flat enum value denoting the quantization level of this synapse model instance
    const SYNAPSE_QUANTIZATION: Self::QuantLevelType;

    /// A nested enum that denotes both the synapse model and the quantization at runtime.
    const NESTED_SYNAPSE_MODEL_AND_QUANTIZATION: SynapseModelTypeAndQuantizationNested;

    /// A flat enum (byte) that denotes both the synapse model and the quantization at runtime.
    /// Useful for some burst engines
    const PACKED_SYNAPSE_MODEL_AND_QUANTIZATION: SynapseModelTypeAndQuantizationPacked = SynapseModelTypeAndQuantizationPacked::from_nested(Self::NESTED_SYNAPSE_MODEL_AND_QUANTIZATION);

    /// All quantizations used by a given synapse model quantization level. Useful for validating
    /// device compatibility. This will also be extended in extensions of this trait
    const USED_QUANTIZATION_LEVELS: &'static [DecimalQuantizationLevel]; // Don't include a default, as we will forget about it
}

/// An enum specific to a synapse model that denotes what synapse model specific quantization preset
/// is using. Runtime counterpart to `SynapseModelQuantization`. Can be packed within a
/// `PackedSynapseModelTypeAndQuantization` for use in burst engines This trait should be implemented
/// for an enum that represents the different quantization presets of the synapse model.
pub trait SynapseModelQuantizationLevel: Clone + Copy + Hash + Eq + PartialEq + Default {
    // Unlike neurons, there is no common shared property all synapses must share
}

