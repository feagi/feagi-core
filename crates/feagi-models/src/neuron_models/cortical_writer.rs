use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::neuron_models::common_enums::NeuronModelTypeAndQuantization;
use crate::neuron_models::neuron_model_traits::NeuronModelQuantizationLevel;

/// Handles reading and writing to neuron and cortical datas in an external friendly form
pub struct CorticalWriter<FIQ: FeagiIndexQuantization> {
    model_type: NeuronModelTypeAndQuantization,

}