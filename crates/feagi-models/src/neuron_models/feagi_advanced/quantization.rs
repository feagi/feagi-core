use crate::neuron_models::neuron_model_traits::NeuronModelQuantizationLevels;
use feagi_data::quantization_levels::cortical_potential_quantization::CorticalPotentialQuantization;
use feagi_data::values::quantizable::{DecimalQuantizationLevel, QuantizedDecimalTrait, QuantizedIndexCountTrait};

#[repr(u8)]
#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
pub enum FeagiAdvancedModelQuantizationLevel {
    #[default]
    Standard32bit = 0b0000_0000,
}

impl NeuronModelQuantizationLevels for FeagiAdvancedModelQuantizationLevel {
    const MODEL_INDEX: u8 = 0b0000_0000;

    fn get_cortical_potential_level(&self) -> DecimalQuantizationLevel {
        DecimalQuantizationLevel::F32
    }

    unsafe fn get_quant_enum_from_quant_bits(quant_bits: u8) -> Self {
        core::mem::transmute(quant_bits)
    }
}

/// The quantization parameters for this neuron model
pub trait FeagiAdvancedModelQuantization: CorticalPotentialQuantization {
    const MODEL_QUANTIZATION_LEVEL: FeagiAdvancedModelQuantizationLevel;

    type NeuronCountdownQuants: QuantizedIndexCountTrait;
    type CorticalLimitAndSnoozeQuants: QuantizedIndexCountTrait;
    type PercentageQuant: QuantizedDecimalTrait; // TODO this will be its own thing later maybe?
    type DegeneracyConstantQuant: QuantizedDecimalTrait;

    // NOTE: Neuron Threshold and leak Quantization should be the CorticalPotentialQuantization
}

//region Discrete Levels
#[derive(Default, Clone, Copy)]
pub struct FeagiAdvancedModelStandard32BitQuant;

impl CorticalPotentialQuantization for FeagiAdvancedModelStandard32BitQuant {
    const CORTICAL_POTENTIAL_QUANTIZATION_LEVEL: DecimalQuantizationLevel = DecimalQuantizationLevel::F32;
    type MembranePotentialQuant = f32;
}

impl FeagiAdvancedModelQuantization for FeagiAdvancedModelStandard32BitQuant {
    const MODEL_QUANTIZATION_LEVEL: FeagiAdvancedModelQuantizationLevel = FeagiAdvancedModelQuantizationLevel::Standard32bit;

    type NeuronCountdownQuants = u16;
    type CorticalLimitAndSnoozeQuants = u16;
    type PercentageQuant = f32;
    type DegeneracyConstantQuant = f32;
}
