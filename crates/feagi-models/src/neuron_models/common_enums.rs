//! These enums are generally used externally (not by burst the burst engine itself) and
//! act largely as a translation between the quant generics of the NPU and the rest of FEAGI

use feagi_data::quantization_levels::cortical_potential_quantization::CorticalMembranePotentialQuantizationLevel;
use crate::neuron_models::feagi_standard::quantization::FeagiStandardModelQuantizationLevel;
use crate::neuron_models::neuron_model_traits::NeuronModelQuantizationLevel;
// TODO macro generation!

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum NeuronModelTypeAndQuantization {
    FeagiStandard(FeagiStandardModelQuantizationLevel),
}

impl NeuronModelTypeAndQuantization {

    pub fn get_membrane_potential_quantization(&self) -> CorticalMembranePotentialQuantizationLevel
    {
        match &self {
            NeuronModelTypeAndQuantization::FeagiStandard(model_quant) => {
                model_quant.get_cortical_potential_level()
            }
        }
    }
}