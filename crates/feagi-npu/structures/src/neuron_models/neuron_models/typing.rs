use crate::neural_processing_unit_data_structures::burst_engine_cluster::burst_engines::data_structures::packed_cortical_descriptor::PackedCorticalDescriptor;
use crate::neuron_models::neuron_models::feagi_standard::quantization::FeagiStandardModelQuantizationLevel;



/// To be added to all neuron model quantization level enums. Enforces them to be able to be
/// 1-1 mappable to 3 bits at the start of a u8. This means that at this time, there can only be
/// 8 neuron models with 8 quantization levels
pub(crate) trait NeuronModelQuantizationBitConversion {
    /// Ensure your model identifier is unique and uses bits 3, 4, and 5
    const NEURON_MODEL_BIT_IDENTIFIER: u8;

    const NEURON_MODEL_QUANT_LEVEL_BITMASK: u8 = 7; // bits 0 1 2
}

// TODO macroize this enum

pub enum NeuronModelTypeAndQuantization
{
    FeagiStandard(FeagiStandardModelQuantizationLevel) // 0u8 (0 0 0)
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

impl From<PackedCorticalDescriptor> for NeuronModelTypeAndQuantization
{
    fn from(value: PackedCorticalDescriptor) -> Self {
        let model_bits: u8 = value.into() & Self::MODEL_TYPE_BITMASK;
        let quant_bits: u8 = value.into() & NeuronModelQuantizationBitConversion::NEURON_MODEL_QUANT_LEVEL_BITMASK;

        // TODO proper logic later!
        NeuronModelTypeAndQuantization::FeagiStandard(
            FeagiStandardModelQuantizationLevel::Standard32bit
        )

    }
}

impl Default for NeuronModelTypeAndQuantization {
    fn default() -> Self {
        NeuronModelTypeAndQuantization::FeagiStandard(FeagiStandardModelQuantizationLevel::default())
    }
}

