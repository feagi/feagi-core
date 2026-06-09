use feagi_structures::feagi_data::quantizable_linear::base_types::{QuantizedDecimalTrait, QuantizedIndexCountTrait};
use feagi_structures::feagi_data::shared_quantization_sets::NeuronModelQuantization;

#[repr(u8)]
#[derive(Default)]
pub enum FeagiStandardModelQuantizationLevel {
    #[default]
    Standard32bit
}

/// The quantization parameters for this neuron model
pub trait FeagiStandardModelQuantization:
NeuronModelQuantization
{
    const QUANTIZATION_LEVEL: FeagiStandardModelQuantizationLevel;
    type NeuronLeakCoefficientQuant: QuantizedDecimalTrait;
    type NeuronConsecutiveFireCountdownQuant: QuantizedIndexCountTrait;
    type NeuronRefractoryCountdownQuant: QuantizedIndexCountTrait;

    // NOTE: No need for padding logic if all 4 below are the same size!

    type CorticalExcitabilityQuant: QuantizedDecimalTrait;
    type CorticalRefractoryPeriodLimitQuant: QuantizedIndexCountTrait;
    type CorticalFireThresholdLimit: QuantizedDecimalTrait;
    type CorticalConsecutiveFireLimit: QuantizedIndexCountTrait;
}

//region Discrete Levels
#[derive(Default)]
pub struct FeagiStandardModelStandard32BitQuant;

impl NeuronModelQuantization for FeagiStandardModelStandard32BitQuant {
    type CorticalPotentialQuant = f32;
}

impl FeagiStandardModelQuantization for FeagiStandardModelStandard32BitQuant {
    const QUANTIZATION_LEVEL: FeagiStandardModelQuantizationLevel = FeagiStandardModelQuantizationLevel::Standard32bit;
    type NeuronLeakCoefficientQuant = f32;
    type NeuronConsecutiveFireCountdownQuant = u16;
    type NeuronRefractoryCountdownQuant = u16;
    type CorticalExcitabilityQuant = f32;
    type CorticalRefractoryPeriodLimitQuant = u32;
    type CorticalFireThresholdLimit = f32;
    type CorticalConsecutiveFireLimit = u32;
}