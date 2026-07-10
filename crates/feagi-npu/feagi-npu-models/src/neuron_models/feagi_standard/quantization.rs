use feagi_data::quantization_levels::cortical_potential_quantization::{
    CorticalMembranePotentialQuantizationLevel, CorticalPotentialQuantization,
};
use feagi_data::values::quantizable::{QuantizedDecimalTrait, QuantizedIndexCountTrait};



#[repr(u8)]
#[derive(Default, Copy, Clone)]
pub enum FeagiStandardModelQuantizationLevel {
    #[default]
    Standard32bit = 0b0000_0000,
}

impl FeagiStandardModelQuantizationLevel {
    pub const MODEL_BITS: u8 = 0b0000_0000;
}


/// The quantization parameters for this neuron model
pub trait FeagiStandardModelQuantization: CorticalPotentialQuantization {
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

impl CorticalPotentialQuantization for FeagiStandardModelStandard32BitQuant {
    const QUANTIZATION_LEVEL: CorticalMembranePotentialQuantizationLevel =
        CorticalMembranePotentialQuantizationLevel::Float32;
    type MembranePotentialQuant = f32;
}

impl FeagiStandardModelQuantization for FeagiStandardModelStandard32BitQuant {
    const QUANTIZATION_LEVEL: FeagiStandardModelQuantizationLevel =
        FeagiStandardModelQuantizationLevel::Standard32bit;

    type NeuronCountdownQuants = u16;
    type CorticalLimitAndSnoozeQuants = u16;
    type PercentageQuant = f32;
    type DegeneracyConstantQuant = f32;


}
