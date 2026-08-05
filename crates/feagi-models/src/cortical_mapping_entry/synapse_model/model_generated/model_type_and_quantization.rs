
// TODO build.rs should generate these enums

use crate::cortical_mapping_entry::synapse_model::models::plastic::PlasticSynapseModelQuantizationLevel;
use crate::cortical_mapping_entry::synapse_model::models::uniform::UniformSynapseModelQuantizationLevel;

/// Describes what Synapse Model is being used without further context. Internally is encoded as an
/// u8 of the same value of the synapse models `MODEL_INDEX` for bitpacking reasons. This makes
/// it easily cross-platform but not easy to use as `NestedSynapseModelTypeAndQuantization`
#[repr(u8)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum SynapseModelType {
    Uniform = 0,
    Plastic = 1
}


/// Describes the synapse model and the synapse model quantization it uses as a nested enum. This
/// makes it convenient to use but not store in cross platform environments
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum SynapseModelTypeAndQuantizationNested {
    Uniform(UniformSynapseModelQuantizationLevel),
    Plastic(PlasticSynapseModelQuantizationLevel)
}

impl SynapseModelTypeAndQuantizationNested {
    /// Init from the packed representation of `PackedSynapseModelTypeAndQuantization`
    pub fn from_packed(packed: SynapseModelTypeAndQuantizationPacked) -> Self {
        packed.to_unpacked()
    }

    pub const fn strip_quantization(&self) -> SynapseModelType {
        match self {
            SynapseModelTypeAndQuantizationNested::Uniform(_) => SynapseModelType::Uniform,
            SynapseModelTypeAndQuantizationNested::Plastic(_) => SynapseModelType::Plastic
        }
    }

    /// Returns the `PackedSynapseModelTypeAndQuantization` of this enum
    pub const fn to_packed(self) -> SynapseModelTypeAndQuantizationPacked {
        SynapseModelTypeAndQuantizationPacked::from_nested(self)
    }
}

/// An enum describing all possible synapse model and model quantizations as a flat list. This is
/// intended for rapid lookups in the NPU and not really general use. The bytes / bits are
/// based on `SYNAPSE_MODEL_TYPE_BITMASK` and `SYNAPSE_MODEL_QUANTIZATION_BITMASK` from
/// `SynapseModelQuantizationLevels`. This is all done in a single byte.
#[allow(non_camel_case_types)]
#[repr(u8)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub enum SynapseModelTypeAndQuantizationPacked {
    #[default]
    Uniform_Standard = (SynapseModelType::Uniform as u8) << 4 | UniformSynapseModelQuantizationLevel::Standard as u8,
    Plastic_Standard = (SynapseModelType::Plastic as u8) << 4 | (PlasticSynapseModelQuantizationLevel::Standard as u8),
}

impl SynapseModelTypeAndQuantizationPacked {
    pub const fn from_nested(nested: SynapseModelTypeAndQuantizationNested) -> Self {
        match nested {
            SynapseModelTypeAndQuantizationNested::Uniform(v) => match v {
                UniformSynapseModelQuantizationLevel::Standard => SynapseModelTypeAndQuantizationPacked::Uniform_Standard,
            },
            SynapseModelTypeAndQuantizationNested::Plastic(v) => match v {
                PlasticSynapseModelQuantizationLevel::Standard => SynapseModelTypeAndQuantizationPacked::Plastic_Standard,
            }
        }
    }

    pub fn to_unpacked(self) -> SynapseModelTypeAndQuantizationNested {
        match self {
            SynapseModelTypeAndQuantizationPacked::Uniform_Standard => {
                SynapseModelTypeAndQuantizationNested::Uniform(UniformSynapseModelQuantizationLevel::Standard)
            }
            SynapseModelTypeAndQuantizationPacked::Plastic_Standard => {
                SynapseModelTypeAndQuantizationNested::Plastic(PlasticSynapseModelQuantizationLevel::Standard)
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

// NOTE: Yes, `PackedSynapseModelTypeAndQuantization` is used mainly in the NPU, but we are
// defining it here since it is following the same pattern in macro generation


