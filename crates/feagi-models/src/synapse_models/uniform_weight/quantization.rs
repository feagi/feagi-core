use crate::synapse_models::synapse_model_traits::synapse_model_data::SynapseModelQuantization;
use feagi_data::values::quantizable::QuantizedDecimalTrait;

#[repr(u8)]
#[derive(Default, Copy, Clone)]
pub enum BasicSynapseModelQuantizationLevel {
    #[default]
    Standard32bit = 0,
}

/// The quantization parameters for this synapse model
pub trait BasicSynapseModelQuantization: SynapseModelQuantization {
    const QUANTIZATION_LEVEL: BasicSynapseModelQuantizationLevel;

    type MultiplierQuant: QuantizedDecimalTrait;
}

//region Discrete Levels
#[derive(Default)]
pub struct BasicSynapseModelStandard32BitQuant;

impl SynapseModelQuantization for BasicSynapseModelStandard32BitQuant {}

impl BasicSynapseModelQuantization for BasicSynapseModelStandard32BitQuant {
    const QUANTIZATION_LEVEL: BasicSynapseModelQuantizationLevel = BasicSynapseModelQuantizationLevel::Standard32bit;
    type MultiplierQuant = f32;
}

//endregion
