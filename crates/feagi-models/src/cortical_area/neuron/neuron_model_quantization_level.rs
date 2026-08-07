use core::hash::Hash;
use feagi_data::values::quantizable::DecimalQuantizationLevel;

/// An enum specific to a neuron model that denotes what neuron model specific quantization preset
/// is using. Runtime counterpart to `NeuronModelQuantization`. Can be packed within a
/// `NeuronModelTypeAndQuantizationPacked` for use in burst engines This trait should be implemented
/// for an enum that represents the different quantization presets of the neuron model.
pub trait NeuronModelQuantizationLevel: Clone + Copy + Hash + Eq + PartialEq + Default {
    /// Calculate the membrane potential level from the given neuron model quantization level. Note
    /// that we do not expect that this be directly encoded in the byte, and should be calculated.
    /// This is alright since this is not used in extremely performance sensitive use cases.
    fn get_membrane_potential_level(&self) -> DecimalQuantizationLevel;
}
