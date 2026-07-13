use crate::neuron_models::neuron_model_traits::NeuronModelQuantizationLevel;
use feagi_data::quantization_levels::cortical_potential_quantization::{
    CorticalMembranePotentialQuantizationLevel, CorticalPotentialCPUQuantization,
};
use feagi_data::values::quantizable::{QuantizedDecimalTrait, QuantizedIndexCountTrait};

#[repr(u8)]
#[derive(Default, Copy, Clone)]
pub enum FeagiStandardModelQuantizationLevel {
    #[default]
    Standard32bit = 0b0000_0000,
}

impl NeuronModelQuantizationLevel for FeagiStandardModelQuantizationLevel {
    const MODEL_INDEX: u8 = 0b0000_0000;

    unsafe fn get_quant_enum_from_quant_bits(quant_bits: u8) -> Self {
        core::mem::transmute(quant_bits)
    }
}

/// The quantization parameters for this neuron model
pub trait FeagiStandardModelQuantization: CorticalPotentialCPUQuantization {
    const QUANTIZATION_LEVEL: FeagiStandardModelQuantizationLevel;

    type NeuronCountdownQuants: QuantizedIndexCountTrait;
    type CorticalLimitAndSnoozeQuants: QuantizedIndexCountTrait;
    type PercentageQuant: QuantizedDecimalTrait; // TODO this will be its own thing later maybe?
    type DegeneracyConstantQuant: QuantizedDecimalTrait;

    // NOTE: Neuron Threshold and leak Quantization should be the CorticalPotentialQuantization
}

//region Discrete Levels
#[derive(Default, Clone, Copy)]
pub struct FeagiStandardModelStandard32BitQuant;

impl CorticalPotentialCPUQuantization for FeagiStandardModelStandard32BitQuant {
    const QUANTIZATION_LEVEL: CorticalMembranePotentialQuantizationLevel = CorticalMembranePotentialQuantizationLevel::Float32;
    type MembranePotentialQuant = f32;
}

impl FeagiStandardModelQuantization for FeagiStandardModelStandard32BitQuant {
    const QUANTIZATION_LEVEL: FeagiStandardModelQuantizationLevel = FeagiStandardModelQuantizationLevel::Standard32bit;

    type NeuronCountdownQuants = u16;
    type CorticalLimitAndSnoozeQuants = u16;
    type PercentageQuant = f32;
    type DegeneracyConstantQuant = f32;
}
