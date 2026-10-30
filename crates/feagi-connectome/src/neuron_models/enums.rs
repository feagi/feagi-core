//! Enums to store types of various models

// TODO this should be macro generated in the future

use feagi_basis::collections::quant_indexed::quant_indexed::QuantContiguousStoreBackend;
use feagi_connectome_neural::neural_basis::neuron_model::data::data_containers::CorticalModelFullStorageData;
use crate::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::neuron_models::invalid_neuron_enum_error::NeuronModelMatchSelectionError;
use crate::neuron_models::core_power::data::{CorePowerFullStorageDataQuantEnum, CorePowerGenomePropertiesDataEnum};
use crate::neuron_models::feagi_advanced::data::{FeagiAdvancedFullStorageDataQuantEnum, FeagiAdvancedGenomePropertiesDataEnum};

// TODO enums need debug, clone

//region Data

pub enum NeuronModelGenomePropertiesEnum<FIQ: FeagiIndexQuantization> {
    CorePower(CorePowerGenomePropertiesDataEnum<FIQ>),
    FeagiAdvanced(FeagiAdvancedGenomePropertiesDataEnum<FIQ>),
}

impl<FIQ: FeagiIndexQuantization> From<CorePowerGenomePropertiesDataEnum<FIQ>> for NeuronModelGenomePropertiesEnum<FIQ>
{
    fn from(value: CorePowerGenomePropertiesDataEnum<FIQ>) -> Self {
        NeuronModelGenomePropertiesEnum::CorePower(value)
    }
}

impl<FIQ: FeagiIndexQuantization> TryFrom<NeuronModelGenomePropertiesEnum<FIQ>> for CorePowerGenomePropertiesDataEnum<FIQ>
{
    type Error = NeuronModelMatchSelectionError;

    fn try_from(value: NeuronModelGenomePropertiesEnum<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NeuronModelGenomePropertiesEnum::CorePower(t) => {
                Ok(t)
            }
            _ => {Err(NeuronModelMatchSelectionError::IncorrectNeuronModel {tried: "", actual: ""})}
        }
    }
}

impl<FIQ: FeagiIndexQuantization> From<FeagiAdvancedGenomePropertiesDataEnum<FIQ>> for NeuronModelGenomePropertiesEnum<FIQ>
{
    fn from(value: FeagiAdvancedGenomePropertiesDataEnum<FIQ>) -> Self {
        NeuronModelGenomePropertiesEnum::FeagiAdvanced(value)
    }
}

impl<FIQ: FeagiIndexQuantization> TryFrom<NeuronModelGenomePropertiesEnum<FIQ>> for FeagiAdvancedGenomePropertiesDataEnum<FIQ>
{
    type Error = NeuronModelMatchSelectionError;

    fn try_from(value: NeuronModelGenomePropertiesEnum<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NeuronModelGenomePropertiesEnum::FeagiAdvanced(t) => {
                Ok(t)
            }
            _ => {Err(NeuronModelMatchSelectionError::IncorrectNeuronModel {tried: "", actual: ""})}
        }
    }
}



pub enum NeuronModelFullStorageDataEnum<FIQ: FeagiIndexQuantization, SB: QuantContiguousStoreBackend> {
    CorePower(CorePowerFullStorageDataQuantEnum<FIQ, SB>),
    FeagiAdvanced(FeagiAdvancedFullStorageDataQuantEnum<FIQ, SB>),
}

impl<FIQ: FeagiIndexQuantization, SB: QuantContiguousStoreBackend> From<CorePowerFullStorageDataQuantEnum<FIQ, SB>> for NeuronModelFullStorageDataEnum<FIQ, SB>
{
    fn from(value: CorePowerFullStorageDataQuantEnum<FIQ, SB>) -> Self {
        NeuronModelFullStorageDataEnum::CorePower(value)
    }
}

impl<FIQ: FeagiIndexQuantization, SB: QuantContiguousStoreBackend> TryFrom<NeuronModelFullStorageDataEnum<FIQ, SB>> for CorePowerFullStorageDataQuantEnum<FIQ, SB>
{
    type Error = NeuronModelMatchSelectionError;

    fn try_from(value: NeuronModelFullStorageDataEnum<FIQ, SB>) -> Result<Self, Self::Error> {
        match value {
            NeuronModelFullStorageDataEnum::CorePower(t) => {
                Ok(t)
            }
            _ => {Err(NeuronModelMatchSelectionError::IncorrectNeuronModel {tried: "", actual: ""})}
        }
    }
}

impl<FIQ: FeagiIndexQuantization, SB: QuantContiguousStoreBackend> From<FeagiAdvancedFullStorageDataQuantEnum<FIQ, SB>> for NeuronModelFullStorageDataEnum<FIQ, SB>
{
    fn from(value: FeagiAdvancedFullStorageDataQuantEnum<FIQ, SB>) -> Self {
        NeuronModelFullStorageDataEnum::FeagiAdvanced(value)
    }
}

impl<FIQ: FeagiIndexQuantization, SB: QuantContiguousStoreBackend> TryFrom<NeuronModelFullStorageDataEnum<FIQ, SB>> for FeagiAdvancedFullStorageDataQuantEnum<FIQ, SB>
{
    type Error = NeuronModelMatchSelectionError;

    fn try_from(value: NeuronModelFullStorageDataEnum<FIQ, SB>) -> Result<Self, Self::Error> {
        match value {
            NeuronModelFullStorageDataEnum::FeagiAdvanced(t) => {
                Ok(t)
            }
            _ => {Err(NeuronModelMatchSelectionError::IncorrectNeuronModel {tried: "", actual: ""})}
        }
    }
}


//endregion



