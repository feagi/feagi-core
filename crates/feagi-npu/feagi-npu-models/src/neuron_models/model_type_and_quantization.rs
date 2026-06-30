use crate::neuron_models::feagi_standard::quantization::FeagiStandardModelQuantizationLevel;

/// To be added to all neuron model quantization level enums. Enforces them to be able to be
/// 
/// 
/// 1-1 mappable to 3 bits at the start of a u8. This means that at this time, there can only be
/// 8 neuron models with 8 quantization levels
pub(crate) trait NeuronModelQuantizationBitConversion {
    /// Ensure your model identifier is unique and uses bits 3, 4, and 5
    const NEURON_MODEL_BIT_IDENTIFIER: u8;

    const NEURON_MODEL_QUANT_LEVEL_BITMASK: u8 = 0b1110_0000;
}

// TODO macroize this enum

pub enum NeuronModelTypeAndQuantization
{
    FeagiStandard(FeagiStandardModelQuantizationLevel)
}

impl NeuronModelTypeAndQuantization
{
    pub(crate) const MODEL_TYPE_BITMASK: u8 = 56; // bits 3, 4, 5

    pub(crate) fn as_neuron_model_and_quantization_u8(self) -> u8 // Get as 6 bits for PackedCorticalDescriptor
    {
        match self {
            NeuronModelTypeAndQuantization::FeagiStandard(quant) =>
                {
                    FeagiStandardModelQuantizationLevel::NEURON_MODEL_BIT_IDENTIFIER & (quant as u8)
                }
        }
    }
}

impl Default for NeuronModelTypeAndQuantization {
    fn default() -> Self {
        NeuronModelTypeAndQuantization::FeagiStandard(FeagiStandardModelQuantizationLevel::default())
    }
}

