use crate::neuron_models::feagi_standard::quantization::FeagiStandardModelQuantizationLevel;

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
