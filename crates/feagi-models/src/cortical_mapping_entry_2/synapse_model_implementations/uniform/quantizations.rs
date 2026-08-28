use crate::cortical_mapping_entry::synapse::synapse_model_quantization::SynapseModelQuantization;
use crate::cortical_mapping_entry::synapse::synapse_model_quantization_level::SynapseModelQuantizationLevel;
use crate::cortical_mapping_entry::synapse_model_implementations::generated_enums::{SynapseModelType, SynapseModelTypeAndQuantizationNested};
use feagi_data::values::quantizable::DecimalQuantizationLevel;

pub trait UniformSynapseModelQuantization: SynapseModelQuantization + Clone + Copy {
    const MODEL_QUANTIZATION_LEVEL: UniformSynapseModelQuantizationLevel;
    // Multiplier quant uses the same quant as the In/ Out for synapse
}

#[derive(Default, Clone, Copy)]
pub struct UniformSynapseModelStandardQuant;

impl UniformSynapseModelQuantization for UniformSynapseModelStandardQuant {
    const MODEL_QUANTIZATION_LEVEL: UniformSynapseModelQuantizationLevel = UniformSynapseModelQuantizationLevel::Standard;
}

impl SynapseModelQuantization for UniformSynapseModelStandardQuant {
    type JunctionPotentialQuant = f32;
    const SYNAPSE_MODEL: SynapseModelType = SynapseModelType::Uniform;
    type QuantLevelType = UniformSynapseModelQuantizationLevel;
    const SYNAPSE_QUANTIZATION: Self::QuantLevelType = UniformSynapseModelQuantizationLevel::Standard;
    const NESTED_SYNAPSE_MODEL_AND_QUANTIZATION: SynapseModelTypeAndQuantizationNested =
        SynapseModelTypeAndQuantizationNested::Uniform(Self::SYNAPSE_QUANTIZATION);
    const USED_QUANTIZATION_LEVELS: &'static [DecimalQuantizationLevel] = &[DecimalQuantizationLevel::F32];
}

/// The quantization used by the Uniform Synapse Model
#[repr(u8)]
#[derive(Debug, Copy, Default, Clone, Hash, PartialEq, Eq)]
pub enum UniformSynapseModelQuantizationLevel {
    #[default]
    Standard = 0,
}

impl SynapseModelQuantizationLevel for UniformSynapseModelQuantizationLevel {
    // TODO copy some properties here
}
