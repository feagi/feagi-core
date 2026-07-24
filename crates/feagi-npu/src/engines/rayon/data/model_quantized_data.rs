use feagi_data::neurons::NeuronMembranePotential;
use feagi_data::quantization_levels::membrane_potential_quantization::MembranePotentialQuantization;
/// Helper structs to make dealing with multiple quantizations / models less annoying
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_models::neuron::model_and_quantization::NestedNeuronModelTypeAndQuantization;
use feagi_models::neuron::models::feagi_advanced::{FeagiAdvancedModelCorticalData, FeagiAdvancedModelNeuronData, FeagiAdvancedModelStandardQuant};
use feagi_models::neuron::neuron_model_data::{NeuronModelCorticalData, NeuronModelNeuronData};
use feagi_models::wrapped_index_collections::{CorticalModelIndexedVector, NeuronModelIndexedVector};

// TODO coincidentally, this can also be used as connectome file storage...



/// Vectors of neuron model data by quantization and model
pub struct NeuronModelData<FIQ: FeagiIndexQuantization> {
    pub cortical_model_feagi_advanced_quant_standard: CorticalModelIndexedVector
    <
        FIQ::CorticalAreaIndexCountQuant,
        FeagiAdvancedModelCorticalData<FeagiAdvancedModelStandardQuant>
    >,

    pub neuron_model_feagi_advanced_quant_standard: NeuronModelIndexedVector
    <
        FIQ::NeuronIndexCountQuant,
        FeagiAdvancedModelNeuronData<FeagiAdvancedModelStandardQuant>,
    >,

    pub neuron_model_feagi_advanced_quant_standard_psp_uniformity: NeuronModelIndexedVector
    <
        FIQ::NeuronIndexCountQuant,
        NeuronMembranePotential<<FeagiAdvancedModelStandardQuant as MembranePotentialQuantization>::MembranePotentialQuant>,
    >,

    // TODO this should be macroized and expanded!
}

impl<FIQ: FeagiIndexQuantization> NeuronModelData<FIQ> {
    pub fn new() -> Self {
        Self {
            cortical_model_feagi_advanced_quant_standard: CorticalModelIndexedVector::new_empty(),
            neuron_model_feagi_advanced_quant_standard: NeuronModelIndexedVector::new_empty(),
            neuron_model_feagi_advanced_quant_standard_psp_uniformity: NeuronModelIndexedVector::new_empty(),
        }
    }
}

// TODO synapse move here!



