//! Enums to store types of various models

// TODO this should be macro generated in the future

use feagi_basis::collections::quant_indexed::quant_indexed::QuantContiguousStoreBackend;
use feagi_connectome_neural::neural_basis::neuron_model::data::data_containers::CorticalModelFullStorageData;
use crate::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::neuron_models::core_power::data::{CorePowerFullStorageDataQuantEnum, CorePowerGenomePropertiesDataEnum};
use crate::neuron_models::feagi_advanced::data::{FeagiAdvancedFullStorageDataQuantEnum, FeagiAdvancedGenomePropertiesDataEnum};

// TODO enums need debug, clone

pub enum NeuronModelFullStorageDataEnum<FIQ: FeagiIndexQuantization, SB: QuantContiguousStoreBackend> {
    CorePower(CorePowerFullStorageDataQuantEnum<FIQ, SB>),
    FeagiAdvanced(FeagiAdvancedFullStorageDataQuantEnum<FIQ, SB>),
}


pub enum NeuronModelGenomePropertiesEnum<FIQ: FeagiIndexQuantization> {
    CorePower(CorePowerGenomePropertiesDataEnum<FIQ>),
    FeagiAdvanced(FeagiAdvancedGenomePropertiesDataEnum<FIQ>),
}



