use feagi_data::values::quantizable::DecimalQuantizationLevel;
use crate::neuron::models::feagi_advanced::FeagiAdvancedModelQuantizationLevel;
use crate::neuron::neuron_model_quantization::NeuronModelQuantizationLevel;

/// Describes what Neuron Model is being used without further context. Internally is encoded as an
/// u8 of the same value of the neuron models `MODEL_INDEX` for bitpacking reasons. This makes
/// it easily cross-platform but not easy to use as `NestedNeuronModelTypeAndQuantization`
#[repr(u8)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum NeuronModelType {
    FeagiAdvanced = 0
}


/// Describes the neuron model and the neuron model quantization it uses as a nested enum. This
/// makes it convenient to use but not store in cross platform environments
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum NeuronModelTypeAndQuantizationNested {
    FeagiAdvanced(FeagiAdvancedModelQuantizationLevel),
}

impl NeuronModelTypeAndQuantizationNested {
    /// Init from the packed representation of `PackedNeuronModelTypeAndQuantization`
    pub fn from_packed(packed: NeuronModelTypeAndQuantizationPacked) -> Self {
        packed.to_unpacked()
    }

    pub const fn strip_quantization(&self) -> NeuronModelType {
        match self {
            NeuronModelTypeAndQuantizationNested::FeagiAdvanced(_) => NeuronModelType::FeagiAdvanced,
        }
    }

    /// Gets the membrane potential quantization via matching through this enum (this information
    /// is not inherently encoded in this struct and needs to be searched for, so do not use
    /// this for high performance requiring functions)
    pub fn get_membrane_potential_quantization(&self) -> DecimalQuantizationLevel {
        match &self {
            NeuronModelTypeAndQuantizationNested::FeagiAdvanced(model_quant) => model_quant.get_membrane_potential_level(),
        }
    }

    /// Returns the `PackedNeuronModelTypeAndQuantization` of this enum
    pub const fn to_packed(self) -> NeuronModelTypeAndQuantizationPacked {
        NeuronModelTypeAndQuantizationPacked::from_nested(self)
    }
}

/// An enum describing all possible neuron model and model quantizations as a flat list. This is
/// intended for rapid lookups in the NPU and not really general use. The bytes / bits are
/// based on `NEURON_MODEL_TYPE_BITMASK` and `NEURON_MODEL_QUANTIZATION_BITMASK` from
/// `NeuronModelQuantizationLevels`. This is all done in a single byte.
#[allow(non_camel_case_types)]
#[repr(u8)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub enum NeuronModelTypeAndQuantizationPacked {
    #[default]
    FeagiAdvanced_Standard = NeuronModelType::FeagiAdvanced as u8 & (FeagiAdvancedModelQuantizationLevel::Standard as u8),
}

impl NeuronModelTypeAndQuantizationPacked {
    pub const fn from_nested(nested: NeuronModelTypeAndQuantizationNested) -> Self {
        match nested {
            NeuronModelTypeAndQuantizationNested::FeagiAdvanced(v) => match v {
                FeagiAdvancedModelQuantizationLevel::Standard => NeuronModelTypeAndQuantizationPacked::FeagiAdvanced_Standard,
            },
        }
    }

    pub fn to_unpacked(self) -> NeuronModelTypeAndQuantizationNested {
        match self {
            NeuronModelTypeAndQuantizationPacked::FeagiAdvanced_Standard => {
                NeuronModelTypeAndQuantizationNested::FeagiAdvanced(FeagiAdvancedModelQuantizationLevel::Standard)
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

