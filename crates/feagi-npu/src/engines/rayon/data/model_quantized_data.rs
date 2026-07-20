/// Helper structs to make dealing with multiple quantizations / models less annoying
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_models::neuron::models::feagi_advanced::data::{FeagiAdvancedModelCorticalData, FeagiAdvancedModelNeuronData};
use feagi_models::neuron::models::feagi_advanced::quantization::FeagiAdvancedModelStandard32BitQuant;
use feagi_npu_common::wrapped_indexes::{CorticalModelIndexedVector, NeuronModelIndexedVector};

// TODO coincidentally, this can also be used as connectome file storage...


/// Vectors of neuron model data by quantization and 
pub struct NeuronModelData<FIQ: FeagiIndexQuantization> {
    pub cortical_model_feagi_advanced_quant_standard_32_bit: CorticalModelIndexedVector
    <
        FIQ::CorticalAreaIndexCountQuant,
        FeagiAdvancedModelCorticalData<FeagiAdvancedModelStandard32BitQuant>
    >,

    pub neuron_model_feagi_advanced_quant_standard_32_bit: NeuronModelIndexedVector
    <
        FIQ::CorticalAreaIndexCountQuant,
        FeagiAdvancedModelNeuronData<FeagiAdvancedModelStandard32BitQuant>,
    >,

    // TODO this should be macroized and expanded!
}

// TODO synapse move here!



