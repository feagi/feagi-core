use crate::neuron::shared::NeuronModelQuantizationLevel;
use feagi_data::feagi_quantization_levels::cortical_potential_quantization::CorticalPotentialQuantization;
use feagi_data::values::quantizable::{DecimalQuantizationLevel, QuantizedDecimalTrait, QuantizedIndexCountTrait};
use crate::neuron::interfacing::model_and_quantization::PackedNeuronModelTypeAndQuantization;

/// The quantization level for the Feagi Advanced Neuron Model
#[repr(u8)]
#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
pub enum FeagiAdvancedModelQuantizationLevel {
    #[default]
    Standard32bit = 0,
}

impl NeuronModelQuantizationLevel for FeagiAdvancedModelQuantizationLevel {
    const MODEL_INDEX: u8 = 0 << Self::NUMBER_BITS_FOR_NEURON_MODEL_TYPE;

    fn get_cortical_potential_level(&self) -> DecimalQuantizationLevel {
        match self {
            FeagiAdvancedModelQuantizationLevel::Standard32bit => DecimalQuantizationLevel::F32
        }
    }

    fn from_packed_neuron_model_and_quant(packed: PackedNeuronModelTypeAndQuantization) -> Self {
        unsafe {core::mem::transmute((packed as u8) & Self::NEURON_MODEL_QUANTIZATION_BITMASK) }
    }
}

/// The quantization parameters for this neuron model
pub trait FeagiAdvancedModelQuantization: CorticalPotentialQuantization {
    const MODEL_QUANTIZATION_LEVEL: FeagiAdvancedModelQuantizationLevel;

    type NeuronCountdownQuants: QuantizedIndexCountTrait;
    type CorticalLimitAndSnoozeQuants: QuantizedIndexCountTrait;
    type PercentageQuant: QuantizedDecimalTrait; // TODO this will be its own thing later maybe?
    type DegeneracyConstantQuant: QuantizedDecimalTrait;
}

//region Discrete Levels
#[derive(Default, Clone, Copy)]
pub struct FeagiAdvancedModelStandard32BitQuant;

impl CorticalPotentialQuantization for FeagiAdvancedModelStandard32BitQuant {
    //const CORTICAL_POTENTIAL_QUANTIZATION_LEVEL: DecimalQuantizationLevel = DecimalQuantizationLevel::F32;
    const USED_QUANTIZATION_LEVELS: &'static [DecimalQuantizationLevel] = &[DecimalQuantizationLevel::F32];
    type MembranePotentialQuant = f32;
}

impl FeagiAdvancedModelQuantization for FeagiAdvancedModelStandard32BitQuant {
    const MODEL_QUANTIZATION_LEVEL: FeagiAdvancedModelQuantizationLevel = FeagiAdvancedModelQuantizationLevel::Standard32bit;

    type NeuronCountdownQuants = u16;
    type CorticalLimitAndSnoozeQuants = u16;
    type PercentageQuant = f32;
    type DegeneracyConstantQuant = f32;
}
