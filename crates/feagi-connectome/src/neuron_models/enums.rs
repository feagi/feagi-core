//! Enums to store types of various models

// TODO this should be macro generated in the future

use feagi_connectome_neural::neural_basis::neuron_model::data::data_containers::CorticalModelFullStorageData;
use crate::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;







pub enum NeuronModelStorageDataEnum<FIQ, PDS> {
    CorePower(CorticalModelFullStorageData<FIQ, CorePower>)
}
