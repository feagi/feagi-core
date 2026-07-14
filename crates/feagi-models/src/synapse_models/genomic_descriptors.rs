//! Structs / Enums that are themselves not quantized or use generics at all. These are the
//! interfaces that act as a bridge from the NPU to the rest of FEAGI
//!
use crate::synapse_models::uniform_weight::quantization::UniformSynapseModelQuantizationLevel;

// TODO a macro should be generating these things

/// Using a nested enum, easily describes the synapse model and the synapse model quantization it uses
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum SynapseModelTypeAndQuantization {
    UniformWeight(UniformSynapseModelQuantizationLevel),
}


