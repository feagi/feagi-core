use crate::synapse_models::shared::synapse_model_data::SynapseModelQuantization;
use feagi_data::values::quantizable::QuantizedDecimalTrait;

#[repr(u8)]
#[derive(Debug, Copy, Default, Clone, Hash, PartialEq, Eq)]
pub enum UniformSynapseModelQuantizationLevel {
    #[default]
    Standard32bit = 0,
}

/// The quantization parameters for this synapse model
pub trait BasicSynapseModelQuantization: SynapseModelQuantization {
    const QUANTIZATION_LEVEL: UniformSynapseModelQuantizationLevel;

    type MultiplierQuant: QuantizedDecimalTrait;
}

//region Discrete Levels

pub struct BasicSynapseModelStandard32BitQuant;

impl SynapseModelQuantization for BasicSynapseModelStandard32BitQuant {}

impl BasicSynapseModelQuantization for BasicSynapseModelStandard32BitQuant {
    const QUANTIZATION_LEVEL: UniformSynapseModelQuantizationLevel = UniformSynapseModelQuantizationLevel::Standard32bit;
    type MultiplierQuant = f32;
}

//endregion
