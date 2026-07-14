//! A Neuron Model Descriptor is a byte that represents a nested enum of the neuron model and the
//! quantization used for it. The reason we need them per Burst Engine device is since different
//! devices support different quantizations, and trying to shove all possible quantizations into
//! a single byte is not reasonable

use crate::neuron_models::feagi_standard::quantization::FeagiStandardModelQuantizationLevel;


// TODO Other device descriptors. We will likely need to go to the sub enums, and instead of
// directly having them be u8's, have a set of consts per device type that any macro will
// use here instead

/// An enum describing all possible neuron model and model quantizations as a flat list, for the
/// CPU specifically
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct NeuronModelCPUDescriptor(u8);

impl NeuronModelCPUDescriptor {
    // TODO macro to generate keys!
    pub const FEAGI_STANDARD_FLOAT_32: Self = Self(FeagiStandardModelQuantizationLevel::Standard32bit as u8);
}

impl Default for NeuronModelCPUDescriptor {
    fn default() -> Self {
        Self::FEAGI_STANDARD_FLOAT_32
    }
}
