// TODO build.rs should generate these enums

use crate::synapse::models::uniform::UniformSynapseModelQuantizationLevel;

/// Describes what Synapse Model is being used without further context. Internally is encoded as an
/// u8 of the same value of the synapse models `MODEL_INDEX` for bitpacking reasons. This makes
/// it easily cross-platform but not easy to use as `NestedSynapseModelTypeAndQuantization`
#[repr(u8)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum SynapseModelType {
    Uniform
}


/// Describes the synapse model and the synapse model quantization it uses as a nested enum. This
/// makes it convenient to use but not store in cross platform environments
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum NestedSynapseModelTypeAndQuantization {
    Uniform(UniformSynapseModelQuantizationLevel),
}

impl NestedSynapseModelTypeAndQuantization {
    /// Init from the packed representation of `PackedSynapseModelTypeAndQuantization`
    pub fn from_packed(packed: PackedSynapseModelTypeAndQuantization) -> Self {
        packed.to_unpacked()
    }

    pub const fn strip_quantization(&self) -> SynapseModelType {
        match self {
            NestedSynapseModelTypeAndQuantization::Uniform(_) => SynapseModelType::Uniform,
        }
    }

    /// Returns the `PackedSynapseModelTypeAndQuantization` of this enum
    pub const fn to_packed(self) -> PackedSynapseModelTypeAndQuantization {
        PackedSynapseModelTypeAndQuantization::from_nested(self)
    }
}


//#[doc(hidden)] // TODO hide when api is stable
/// An enum describing all possible synapse model and model quantizations as a flat list. This is
/// intended for rapid lookups in the NPU and not really general use. The bytes / bits are
/// based on `SYNAPSE_MODEL_TYPE_BITMASK` and `SYNAPSE_MODEL_QUANTIZATION_BITMASK` from
/// `SynapseModelQuantizationLevels`. This is all done in a single byte.
#[allow(non_camel_case_types)]
#[repr(u8)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub enum PackedSynapseModelTypeAndQuantization {
    #[default]
    Uniform_Standard = SynapseModelType::Uniform as u8 & (UniformSynapseModelQuantizationLevel::Standard as u8),
}

impl PackedSynapseModelTypeAndQuantization {
    pub const fn from_nested(nested: NestedSynapseModelTypeAndQuantization) -> Self {
        match nested {
            NestedSynapseModelTypeAndQuantization::Uniform(v) => match v {
                UniformSynapseModelQuantizationLevel::Standard => PackedSynapseModelTypeAndQuantization::Uniform_Standard,
            },
        }
    }

    pub fn to_unpacked(self) -> NestedSynapseModelTypeAndQuantization {
        match self {
            PackedSynapseModelTypeAndQuantization::Uniform_Standard => {
                NestedSynapseModelTypeAndQuantization::Uniform(UniformSynapseModelQuantizationLevel::Standard)
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


