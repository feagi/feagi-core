use std::hash::Hash;
use feagi_data::quantization_levels::membrane_potential_quantization::MembranePotentialQuantization;
use feagi_data::values::quantizable::DecimalQuantizationLevel;

/// Common root trait shared by all Neuron Model Quantizations. This trait should be extended
/// by the given neuron model to add any quantization parameters for their given data
pub trait CorticalAreaModelQuantization: MembranePotentialQuantization + Default + Clone {
    // /// A flat enum value denoting what type of cortical area model this cortical area 
    // /// model instance is
    //const NEURON_MODEL: NeuronModelType;

    // /// The type of enum that can denote the quantization level of this cortical model
    //type QuantLevelType: CorticalAreaModelQuantizationLevel;

    // /// A flat enum value denoting the quantization level of this cortical model instance
    // const NEURON_QUANTIZATION: Self::QuantLevelType;

    // /// A nested enum that denotes both the cortical model and the quantization at runtime.
    // const NESTED_NEURON_MODEL_AND_QUANTIZATION: NeuronModelTypeAndQuantizationNested;

    // /// A flat enum (byte) that denotes both the cortical model and the quantization at runtime.
    // /// Useful for some burst engines
    // const PACKED_NEURON_MODEL_AND_QUANTIZATION: NeuronModelTypeAndQuantizationPacked =
    //     NeuronModelTypeAndQuantizationPacked::from_nested(Self::NESTED_NEURON_MODEL_AND_QUANTIZATION);

    // /// All quantizations used by a given cortical model quantization level. Useful for validating
    // /// device compatibility. This will also be extended in extensions of this trait
    // const USED_DECIMAL_QUANTIZATION_LEVELS: &'static [DecimalQuantizationLevel]; // Don't include a default, as we then forget about it
}

pub trait CorticalAreaModelQuantizationLevel: Clone + Copy + Hash + Eq + PartialEq + Default {
    // /// Calculate the membrane potential level from the given cortical model quantization level. Note
    // /// that we do not expect that this be directly encoded in the byte, and should be calculated.
    // /// This is alright since this is not used in extremely performance sensitive use cases.
    // fn get_membrane_potential_level(&self) -> DecimalQuantizationLevel;
}