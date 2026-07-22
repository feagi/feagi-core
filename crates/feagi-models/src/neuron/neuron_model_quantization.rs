use core::hash::Hash;
use crate::neuron::model_and_quantization::{NestedNeuronModelTypeAndQuantization, NeuronModelType, PackedNeuronModelTypeAndQuantization};
use feagi_data::quantization_levels::membrane_potential_quantization::MembranePotentialQuantization;
use feagi_data::values::quantizable::DecimalQuantizationLevel;

/// Common root trait shared by all neuron Model Quantizations. This trait should be extended
/// by the given neuron model to add any quantization parameters for their given data
pub trait NeuronModelQuantization: MembranePotentialQuantization {
    /// A flat enum value denoting what type of neuron model this neuron model instance is
    const NEURON_MODEL: NeuronModelType;

    /// The type of enum that can denote the quantization level of this neuron model
    type QuantLevelType: NeuronModelQuantizationLevel;

    /// A flat enum value denoting the quantization level of this neuron model instance
    const NEURON_QUANTIZATION: Self::QuantLevelType;

    /// A nested enum that denotes both the neuron model and the quantization at runtime.
    const NESTED_NEURON_MODEL_AND_QUANTIZATION: NestedNeuronModelTypeAndQuantization;

    /// A flat enum (byte) that denotes both the neuron model and the quantization at runtime.
    /// Useful for some burst engines
    const PACKED_NEURON_MODEL_AND_QUANTIZATION: PackedNeuronModelTypeAndQuantization = PackedNeuronModelTypeAndQuantization::from_nested(Self::NESTED_NEURON_MODEL_AND_QUANTIZATION);

    /// All quantizations used by a given neuron model quantization level. Useful for validating
    /// device compatibility. This will also be extended in extensions of this trait
    const USED_DECIMAL_QUANTIZATION_LEVELS: &'static [DecimalQuantizationLevel]; // Don't include a default, as we then forget about it
}

/// An enum specific to a neuron model that denotes what neuron model specific quantization preset
/// is using. Runtime counterpart to `NeuronModelQuantization`. Can be packed within a
/// `PackedNeuronModelTypeAndQuantization` for use in burst engines This trait should be implemented
/// for an enum that represents the different quantization presets of the neuron model.
pub trait NeuronModelQuantizationLevel: Clone + Copy + Hash + Eq + PartialEq + Default {
    /// The number of bits dedicated to the model type
    //const NUMBER_BITS_FOR_NEURON_MODEL_TYPE: u8 = 5;

    /// The number of bits dedicated to the quantization level
    //const NUMBER_BITS_FOR_NEURON_MODEL_QUANTIZATION: u8 = 8 - Self::NUMBER_BITS_FOR_NEURON_MODEL_TYPE; // 3
    //const NEURON_MODEL_TYPE_BITMASK: u8 = 255 << Self::NUMBER_BITS_FOR_NEURON_MODEL_QUANTIZATION; // 0b1111_1000
    //const NEURON_MODEL_QUANTIZATION_BITMASK: u8 = 255 >> Self::NUMBER_BITS_FOR_NEURON_MODEL_TYPE; // 0b0000_0111

    /// The index of the model. Make sure it does not conflict with other models
    //const MODEL_INDEX: u8;

    /// Calculate the membrane potential level from the given neuron model quantization level. Note
    /// that we do not expect that this be directly encoded in the byte, and should be calculated.
    /// This is alright since this is not used in extremely performance sensitive use cases.
    fn get_membrane_potential_level(&self) -> DecimalQuantizationLevel;
}
