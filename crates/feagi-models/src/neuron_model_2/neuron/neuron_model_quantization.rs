use crate::neuron_model::neuron::neuron_model_quantization_level::NeuronModelQuantizationLevel;
use crate::neuron_model::neuron_model_implementations::generated_enums::{
    NeuronModelType, NeuronModelTypeAndQuantizationNested, NeuronModelTypeAndQuantizationPacked,
};
use feagi_data::quantization_levels::membrane_potential_quantization::MembranePotentialQuantization;
use feagi_data::values::quantizable::DecimalQuantizationLevel;

/// Common root trait shared by all Neuron Model Quantizations. This trait should be extended
/// by the given neuron model to add any quantization parameters for their given data
pub trait NeuronModelQuantization: MembranePotentialQuantization + Default + Clone {
    /// A flat enum value denoting what type of neuron model this neuron model instance is
    const NEURON_MODEL: NeuronModelType;

    /// The type of enum that can denote the quantization level of this neuron model
    type QuantLevelType: NeuronModelQuantizationLevel;

    /// A flat enum value denoting the quantization level of this neuron model instance
    const NEURON_QUANTIZATION: Self::QuantLevelType;

    /// A nested enum that denotes both the neuron model and the quantization at runtime.
    const NESTED_NEURON_MODEL_AND_QUANTIZATION: NeuronModelTypeAndQuantizationNested;

    /// A flat enum (byte) that denotes both the neuron model and the quantization at runtime.
    /// Useful for some burst engines
    const PACKED_NEURON_MODEL_AND_QUANTIZATION: NeuronModelTypeAndQuantizationPacked =
        NeuronModelTypeAndQuantizationPacked::from_nested(Self::NESTED_NEURON_MODEL_AND_QUANTIZATION);

    /// All quantizations used by a given neuron model quantization level. Useful for validating
    /// device compatibility. This will also be extended in extensions of this trait
    const USED_DECIMAL_QUANTIZATION_LEVELS: &'static [DecimalQuantizationLevel]; // Don't include a default, as we then forget about it
}
