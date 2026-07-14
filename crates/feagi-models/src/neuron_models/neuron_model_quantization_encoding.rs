//! These enums are generally used externally (not by burst the burst engine itself) and
//! act largely as a translation between the quant generics of the NPU and the rest of FEAGI

use feagi_data::values::quantizable::DecimalQuantizationLevel;
use crate::neuron_models::feagi_advanced::quantization::FeagiAdvancedModelQuantizationLevel;
use crate::neuron_models::neuron_model_traits::NeuronModelQuantizationLevels;
// TODO macro generation!

/// Using a nested enum, easily describes the neuron model and the neuron model quantization it uses
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum NeuronModelTypeAndQuantization {
    FeagiAdvanced(FeagiAdvancedModelQuantizationLevel),
}

impl NeuronModelTypeAndQuantization {

    /// Gets the membrane potential quantization via matching through this enum (this information
    /// is not inherently encoded in this struct and needs to be searched for, so do not use
    /// this for high performance requiring functions)
    pub fn get_membrane_potential_quantization(&self) -> DecimalQuantizationLevel
    {
        match &self {
            NeuronModelTypeAndQuantization::FeagiAdvanced(model_quant) => {
                model_quant.get_cortical_potential_level()
            }
        }
    }
}


