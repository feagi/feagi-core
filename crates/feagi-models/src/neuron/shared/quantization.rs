use feagi_data::values::quantizable::DecimalQuantizationLevel;
use crate::neuron::interfacing::model_and_quantization::PackedNeuronModelTypeAndQuantization;

/// Defines the quantization of neurons within a cortical area
pub trait NeuronModelQuantizationLevel: Clone + Copy {
    /// The number of bits dedicated to the model type
    const NUMBER_BITS_FOR_NEURON_MODEL_TYPE: u8 = 5;
    /// The number of bits dedicated to the quantization level
    const NUMBER_BITS_FOR_NEURON_MODEL_QUANTIZATION: u8 = 8 - Self::NUMBER_BITS_FOR_NEURON_MODEL_TYPE; // 3
    const NEURON_MODEL_TYPE_BITMASK: u8 = 255 << Self::NUMBER_BITS_FOR_NEURON_MODEL_QUANTIZATION; // 0b1111_1000
    const NEURON_MODEL_QUANTIZATION_BITMASK: u8 = 255 >> Self::NUMBER_BITS_FOR_NEURON_MODEL_TYPE; // 0b0000_0111
    
    /// The index of the model. Make sure it does not conflict with other models
    const MODEL_INDEX: u8;
    
    /// Calculate the cortical potential level from the given neuron model quantization level. Note
    /// that we do not expect that this be directly encoded in the byte, and should be calculated.
    /// This is alright since this is not used in extremely performance sensitive use cases.
    fn get_cortical_potential_level(&self) -> DecimalQuantizationLevel;
    
    /// Convert directly from a 'PackedNeuronModelTypeAndQuantization'. Will be safe since
    /// 'PackedNeuronModelTypeAndQuantization' is controlled
    fn from_packed_neuron_model_and_quant(packed: PackedNeuronModelTypeAndQuantization) -> Self;
}

// NOTE: we dont need to define a "NeuronModelQuantization" as all the model specific trait
// extend off of `CorticalPotentialQuantization`