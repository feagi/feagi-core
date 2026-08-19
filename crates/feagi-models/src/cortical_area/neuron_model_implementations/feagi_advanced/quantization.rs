use crate::cortical_area::neuron::neuron_model_quantization::NeuronModelQuantization;
use crate::cortical_area::neuron::neuron_model_quantization_level::NeuronModelQuantizationLevel;
use crate::cortical_area::neuron_model_implementations::generated_enums::{NeuronModelType, NeuronModelTypeAndQuantizationNested};
use feagi_data::quantization_levels::membrane_potential_quantization::MembranePotentialQuantization;
use feagi_data::values::quantizable::{DecimalQuantizationLevel, QuantizedDecimalTrait, QuantizedUnsignedIntegerTrait};
use half::bf16;

pub trait FeagiAdvancedModelQuantization: NeuronModelQuantization {
    const MODEL_QUANTIZATION_LEVEL: FeagiAdvancedModelQuantizationLevel;

    type NeuronCountdownQuants: QuantizedUnsignedIntegerTrait;
    type CorticalLimitAndSnoozeQuants: QuantizedUnsignedIntegerTrait;
    type PercentageQuant: QuantizedDecimalTrait;
    type DegeneracyConstantQuant: QuantizedDecimalTrait;
}

//region Discrete Levels

/// The default quantization level for Feagi Advanced
#[derive(Debug, Default, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct FeagiAdvancedModelStandardQuant;

impl FeagiAdvancedModelQuantization for FeagiAdvancedModelStandardQuant {
    const MODEL_QUANTIZATION_LEVEL: FeagiAdvancedModelQuantizationLevel = FeagiAdvancedModelQuantizationLevel::Standard;

    type NeuronCountdownQuants = u16;
    type CorticalLimitAndSnoozeQuants = u16;
    type PercentageQuant = bf16;
    type DegeneracyConstantQuant = f32;
}

impl NeuronModelQuantization for FeagiAdvancedModelStandardQuant {
    const NEURON_MODEL: NeuronModelType = NeuronModelType::FeagiAdvanced;
    type QuantLevelType = FeagiAdvancedModelQuantizationLevel;
    const NEURON_QUANTIZATION: Self::QuantLevelType = FeagiAdvancedModelQuantizationLevel::Standard;
    const NESTED_NEURON_MODEL_AND_QUANTIZATION: NeuronModelTypeAndQuantizationNested =
        NeuronModelTypeAndQuantizationNested::FeagiAdvanced(Self::NEURON_QUANTIZATION);
    const USED_DECIMAL_QUANTIZATION_LEVELS: &'static [DecimalQuantizationLevel] = &[DecimalQuantizationLevel::BF16, DecimalQuantizationLevel::F32];
}

impl MembranePotentialQuantization for FeagiAdvancedModelStandardQuant {
    type MembranePotentialQuant = f32;
}

//endregion

// TODO macro for implementing NeuronModelQuantizationLevel on FeagiAdvancedModelQuantizationLevel

#[repr(u8)]
#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
pub enum FeagiAdvancedModelQuantizationLevel {
    #[default]
    Standard = 0,
}

impl NeuronModelQuantizationLevel for FeagiAdvancedModelQuantizationLevel {
    fn get_membrane_potential_level(&self) -> DecimalQuantizationLevel {
        match self {
            FeagiAdvancedModelQuantizationLevel::Standard => DecimalQuantizationLevel::F32,
        }
    }

    // TODO copy some properties here
}
