use std::hash::Hash;

/// An enum specific to a synapse model that denotes what synapse model specific quantization preset
/// is using. Runtime counterpart to `SynapseModelQuantization`. Can be packed within a
/// `PackedSynapseModelTypeAndQuantization` for use in burst engines This trait should be implemented
/// for an enum that represents the different quantization presets of the synapse model.
pub trait SynapseModelQuantizationLevel: Clone + Copy + Hash + Eq + PartialEq + Default {
    // TODO when SynapseModelQuantization gets to inherit the junction trait, expose it here like with neuron model
}