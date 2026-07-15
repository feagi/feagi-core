use feagi_data::values::quantizable::{DecimalQuantizationLevel, QuantizedDecimalTrait};
use crate::synapse_models::interfacing::model_and_quantization::PackedSynapseModelTypeAndQuantization;
use crate::synapse_models::shared::quantization::{SynapseModelQuantization, SynapseModelQuantizationLevel};

/// The quantization used by the Uniform Synapse Model
#[repr(u8)]
#[derive(Debug, Copy, Default, Clone, Hash, PartialEq, Eq)]
pub enum UniformSynapseModelQuantizationLevel {
    #[default]
    Standard32bit = 0,
}

impl SynapseModelQuantizationLevel for UniformSynapseModelQuantizationLevel {
    const MODEL_INDEX: u8 = 0;

    fn from_packed_synapse_model_and_quant(packed: PackedSynapseModelTypeAndQuantization) -> Self {
        unsafe {core::mem::transmute((packed as u8) & Self::SYNAPSE_MODEL_QUANTIZATION_BITMASK) }
    }
}

/// The quantization parameters for this synapse model
pub trait UniformSynapseModelQuantization: SynapseModelQuantization {
    const QUANTIZATION_LEVEL: UniformSynapseModelQuantizationLevel;

    type MultiplierQuant: QuantizedDecimalTrait;
}

//region Discrete Levels

#[derive(Debug, Clone, Copy)]
pub struct UniformSynapseModelStandard32BitQuant;

impl SynapseModelQuantization for UniformSynapseModelStandard32BitQuant {
    const USED_QUANTIZATION_LEVELS: &'static [DecimalQuantizationLevel] = &[DecimalQuantizationLevel::F32];
}

impl UniformSynapseModelQuantization for UniformSynapseModelStandard32BitQuant {
    const QUANTIZATION_LEVEL: UniformSynapseModelQuantizationLevel = UniformSynapseModelQuantizationLevel::Standard32bit;
    type MultiplierQuant = f32;
}

//endregion
