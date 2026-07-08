use crate::neuron_models::model_type_and_quantization::NeuronModelQuantizationBitConversion;
use feagi_data::quantization_levels::cortical_potential_quantization::CorticalPotentialQuantizationFloat32;
use feagi_data::quantization_levels::extendable_quantizations::NeuronModelQuantization;
use feagi_data::values::quantizable::{QuantizedDecimalTrait, QuantizedIndexCountTrait};

// TODO some sort of way to go from this enum to the right type

// where the first 5 bits are the model and the last 4 are the quantization level

#[repr(u8)]
#[derive(Default, Copy, Clone)]
pub enum FeagiStandardModelQuantizationLevel {
    #[default]
    Standard32bit = 0b0000_0000,
}

impl NeuronModelQuantizationBitConversion for FeagiStandardModelQuantizationLevel {
    // Feagi Standard shall be 0 0
    const NEURON_MODEL_BIT_IDENTIFIER: u8 = 0b0000_0000;
}

/// The quantization parameters for this neuron model
pub trait FeagiStandardModelQuantization: NeuronModelQuantization {
    const QUANTIZATION_LEVEL: FeagiStandardModelQuantizationLevel;

    type NeuronLeakCoefficientQuant: QuantizedDecimalTrait;
    type NeuronConsecutiveFireCountdownQuant: QuantizedIndexCountTrait;
    type NeuronRefractoryCountdownQuant: QuantizedIndexCountTrait;

    type CorticalExcitabilityQuant: QuantizedDecimalTrait;
    type CorticalRefractoryPeriodLimitQuant: QuantizedIndexCountTrait;
    type CorticalFireThreshold: QuantizedDecimalTrait;
    type CorticalConsecutiveFireLimit: QuantizedIndexCountTrait;

    type CorticalSnoozePeriod: QuantizedIndexCountTrait;

    type CorticalDegeneracyConstant: QuantizedDecimalTrait;
}

//region Discrete Levels
#[derive(Default)]
pub struct FeagiStandardModelStandard32BitQuant;

impl NeuronModelQuantization for FeagiStandardModelStandard32BitQuant {
    type CorticalPotentialQuant = CorticalPotentialQuantizationFloat32;
}

impl FeagiStandardModelQuantization for FeagiStandardModelStandard32BitQuant {
    const QUANTIZATION_LEVEL: FeagiStandardModelQuantizationLevel =
        FeagiStandardModelQuantizationLevel::Standard32bit;
    type NeuronLeakCoefficientQuant = f32;
    type NeuronConsecutiveFireCountdownQuant = u16;
    type NeuronRefractoryCountdownQuant = u16;
    type CorticalExcitabilityQuant = f32;
    type CorticalRefractoryPeriodLimitQuant = u32;
    type CorticalFireThreshold = f32;
    type CorticalConsecutiveFireLimit = u32;
    type CorticalSnoozePeriod = u32;
    type CorticalDegeneracyConstant = f32;
}
