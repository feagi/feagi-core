use feagi_models::neuron_models::feagi_advanced::quantization::FeagiAdvancedModelQuantizationLevel;
use feagi_models::neuron_models::neuron_model_quantization_encoding::NeuronModelTypeAndQuantization;

// TODO macro to generate keys!

/// An enum describing all possible neuron model and model quantizations as a flat list. This is
/// intended for rapid lookups in the NPU
#[allow(non_camel_case_types)]
#[repr(u8)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub enum PackedNeuronModelTypeAndQuantization {
    #[default]
    FeagiAdvanced_F32 = 0,
}

impl PackedNeuronModelTypeAndQuantization {
    
    /// Quickly convert from a byte without safety checking. Note that an invalid byte will cause
    /// undefined behavior!
    pub unsafe fn from_byte_unchecked(byte: u8) -> Self {
        core::mem::transmute(byte)
    }
}

impl Into<NeuronModelTypeAndQuantization> for PackedNeuronModelTypeAndQuantization {
    fn into(self) -> NeuronModelTypeAndQuantization {
        match self {
            PackedNeuronModelTypeAndQuantization::FeagiAdvanced_F32 => {
                NeuronModelTypeAndQuantization::FeagiAdvanced(FeagiAdvancedModelQuantizationLevel::Standard32bit)
            }
        }
    }
}

impl From<NeuronModelTypeAndQuantization> for PackedNeuronModelTypeAndQuantization {
    fn from(value: NeuronModelTypeAndQuantization) -> Self {
        match value {
            NeuronModelTypeAndQuantization::FeagiAdvanced(v) => match v {
                FeagiAdvancedModelQuantizationLevel::Standard32bit => PackedNeuronModelTypeAndQuantization::FeagiAdvanced_F32,
            },
        }
    }
}
