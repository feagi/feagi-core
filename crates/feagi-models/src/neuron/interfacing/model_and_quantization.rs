use crate::neuron::models::feagi_advanced::quantization::FeagiAdvancedModelQuantizationLevel;
use crate::neuron::models_shared::NeuronModelQuantizationLevel;
use feagi_data::values::quantizable::DecimalQuantizationLevel;

// TODO build.rs should generate these enums

/// Describes what Neuron Model is being used without further context. Internally is encoded as an
/// u8 of the same value of the neuron models `MODEL_INDEX` for bitpacking reasons.
#[repr(u8)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum NeuronModelType {
    FeagiAdvanced = FeagiAdvancedModelQuantizationLevel::MODEL_INDEX,
}

/// Describes the neuron model and the neuron model quantization it uses as a nested enum
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum NeuronModelTypeAndQuantization {
    FeagiAdvanced(FeagiAdvancedModelQuantizationLevel),
}

impl NeuronModelTypeAndQuantization {
    pub fn strip_quantization(&self) -> NeuronModelType {
        match self {
            NeuronModelTypeAndQuantization::FeagiAdvanced(_) => NeuronModelType::FeagiAdvanced,
        }
    }

    /// Gets the membrane potential quantization via matching through this enum (this information
    /// is not inherently encoded in this struct and needs to be searched for, so do not use
    /// this for high performance requiring functions)
    pub fn get_membrane_potential_quantization(&self) -> DecimalQuantizationLevel {
        match &self {
            NeuronModelTypeAndQuantization::FeagiAdvanced(model_quant) => model_quant.get_cortical_potential_level(),
        }
    }
}

//#[doc(hidden)] // TODO hide when api is stable
/// An enum describing all possible neuron model and model quantizations as a flat list. This is
/// intended for rapid lookups in the NPU and not really general use. The bytes / bits are
/// based on `NEURON_MODEL_TYPE_BITMASK` and `NEURON_MODEL_QUANTIZATION_BITMASK` from
/// `NeuronModelQuantizationLevels`. This is all done ina  single byte.
#[allow(non_camel_case_types)]
#[repr(u8)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub enum PackedNeuronModelTypeAndQuantization {
    #[default]
    FeagiAdvanced_Standard32Bit = FeagiAdvancedModelQuantizationLevel::MODEL_INDEX & (FeagiAdvancedModelQuantizationLevel::Standard32bit as u8),
}

impl PackedNeuronModelTypeAndQuantization {
    pub fn from_unpacked(value: NeuronModelTypeAndQuantization) -> Self {
        match value {
            NeuronModelTypeAndQuantization::FeagiAdvanced(v) => match v {
                FeagiAdvancedModelQuantizationLevel::Standard32bit => PackedNeuronModelTypeAndQuantization::FeagiAdvanced_Standard32Bit,
            },
        }
    }

    pub fn to_unpacked(self) -> NeuronModelTypeAndQuantization {
        match self {
            PackedNeuronModelTypeAndQuantization::FeagiAdvanced_Standard32Bit => {
                NeuronModelTypeAndQuantization::FeagiAdvanced(FeagiAdvancedModelQuantizationLevel::Standard32bit)
            }
        }
    }
    
    pub fn to_byte(self) -> u8 {
        self as u8
    }

    /// Quickly convert from a byte without safety checking. Note that an invalid byte will cause
    /// undefined behavior!
    pub unsafe fn from_byte_unchecked(byte: u8) -> Self {
        core::mem::transmute(byte)
    }
}

// NOTE: Yes, `PackedNeuronModelTypeAndQuantization` is used mainly in the NPU, but we are
// defining it here since it is following the same pattern in macro generation