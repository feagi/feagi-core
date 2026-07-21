use feagi_data::quantization_levels::cortical_potential_quantization::CorticalPotentialQuantization;
/// Helper structs to make dealing with multiple quantizations / models less annoying
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_models::neuron::interfacing::model_and_quantization::NeuronModelTypeAndQuantization;
use feagi_models::neuron::models::feagi_advanced::data::{FeagiAdvancedModelCorticalData, FeagiAdvancedModelNeuronData};
use feagi_models::neuron::models::feagi_advanced::quantization::{FeagiAdvancedModelQuantizationLevel, FeagiAdvancedModelStandard32BitQuant};
use feagi_models::neuron::models_shared::data::{NeuronModelCorticalData, NeuronModelNeuronData};
use feagi_models::wrapped_indexes::{CorticalModelIndexedVector, NeuronModelIndexedVector};

// TODO coincidentally, this can also be used as connectome file storage...


/// Vectors of neuron model data by quantization and model
pub struct NeuronModelData<FIQ: FeagiIndexQuantization> {
    pub cortical_model_feagi_advanced_quant_standard_32_bit: CorticalModelIndexedVector
    <
        FIQ::CorticalAreaIndexCountQuant,
        FeagiAdvancedModelCorticalData<FeagiAdvancedModelStandard32BitQuant>
    >,

    pub neuron_model_feagi_advanced_quant_standard_32_bit: NeuronModelIndexedVector
    <
        FIQ::NeuronIndexCountQuant,
        FeagiAdvancedModelNeuronData<FeagiAdvancedModelStandard32BitQuant>,
    >,

    // TODO this should be macroized and expanded!
}

impl<FIQ: FeagiIndexQuantization> NeuronModelData<FIQ> {
    pub fn new() -> Self {
        Self {
            cortical_model_feagi_advanced_quant_standard_32_bit: CorticalModelIndexedVector::new_empty(),
            neuron_model_feagi_advanced_quant_standard_32_bit: NeuronModelIndexedVector::new_empty(),
        }
    }
    
    pub fn get_quant_cortical_data<CPQ: CorticalPotentialQuantization, NMCD: NeuronModelCorticalData<CPQ>>(&self) -> &NMCD
    {
        match NMCD::LEVEL {
            NeuronModelTypeAndQuantization::FeagiAdvanced(quant) => {
                match quant { 
                    FeagiAdvancedModelQuantizationLevel::Standard32bit => { &self.cortical_model_feagi_advanced_quant_standard_32_bit} 
                }
            }
        }
    }

    pub fn get_quant_neuron_data<CPQ: CorticalPotentialQuantization, NMND: NeuronModelNeuronData<CPQ>>(&self) -> &NMND
    {
        match NMND::LEVEL {
            NeuronModelTypeAndQuantization::FeagiAdvanced(quant) => {
                match quant {
                    FeagiAdvancedModelQuantizationLevel::Standard32bit => { &self.neuron_model_feagi_advanced_quant_standard_32_bit}
                }
            }
        }
    }
    
    
}

// TODO synapse move here!



