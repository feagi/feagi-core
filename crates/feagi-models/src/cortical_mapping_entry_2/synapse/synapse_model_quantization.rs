use crate::cortical_mapping_entry::synapse::synapse_model_quantization_level::SynapseModelQuantizationLevel;
use crate::cortical_mapping_entry::synapse_model_implementations::generated_enums::{
    SynapseModelType, SynapseModelTypeAndQuantizationNested, SynapseModelTypeAndQuantizationPacked,
};
use feagi_data::values::quantizable::{DecimalQuantizationLevel, QuantizedDecimalTrait};

/// Common root trait shared by all Synapse Model Quantizations. This trait should be extended
/// by the given synapse model to add any quantization parameters for their given data
pub trait SynapseModelQuantization: Clone + Default {
    // TODO lets extend off this instead of having this as a sub type, to keep consistent with neurons
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
    const PACKED_SYNAPSE_MODEL_AND_QUANTIZATION: SynapseModelTypeAndQuantizationPacked =
        SynapseModelTypeAndQuantizationPacked::from_nested(Self::NESTED_SYNAPSE_MODEL_AND_QUANTIZATION);

    /// All quantizations used by a given synapse model quantization level. Useful for validating
    /// device compatibility. This will also be extended in extensions of this trait
    const USED_QUANTIZATION_LEVELS: &'static [DecimalQuantizationLevel]; // Don't include a default, as we will forget about it
}
