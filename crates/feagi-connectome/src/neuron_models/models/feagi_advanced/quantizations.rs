use serde::{Deserialize, Serialize};
use crate::neuron_models::models::feagi_advanced::{NMFeagiAdvancedDataCorticalInternal, NMFeagiAdvancedDataCorticalProperties, NMFeagiAdvancedDataCorticalScratch, NMFeagiAdvancedDataCorticalVariance, NMFeagiAdvancedDataNeuronInternal, NMFeagiAdvancedDataNeuronScratch, NMFeagiAdvancedDataNeuronVariance, NMFeagiAdvancedTrait};
use crate::neuron_models::models::{NMKind, NMModelKind, NMNeurogenesisGeneratorModeled, NMNeurogenesisStorageModeled, NMNeurogenesisStorageOrGeneratorModeled, NeuronModelEnumConversionError};
use crate::neuron_models::models::{ProxyNMNeuroGenesisFromGenerator, ProxyNMNeuroGenesisFromStorage, ProxyNMNeurogenesisFromStorageOrByGenerator};
use crate::neuron_models::neurogenesis_generator_uniform_defaults::NeurogenesisGeneratorUniformDefault;
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::voxel::NeuronLayoutTagVoxel;
use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome_neural::neural_basis::quantization_levels::{FeagiIndexQuantization, MembranePotentialQuantization};


//region enums

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
/// Describes the quantization of the Feagi Advanced Neuron Model
pub enum NMFeagiAdvancedQuantKind {
    Standard32Bit,
}

/// Allows Neurogenesis of the Feagi Advanced Neuron Model by its generator for a given quantization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum NMFeagiAdvancedNeurogenesisGeneratorQuanted<FIQ: FeagiIndexQuantization> {
    Standard32Bit(ProxyNMNeuroGenesisFromGenerator<FIQ, NMFeagiAdvancedQuantStandard32Bit>)
}

impl<FIQ: FeagiIndexQuantization> NMFeagiAdvancedNeurogenesisGeneratorQuanted<FIQ> {
    /// The quantization of the generator held
    pub const fn quant_kind(&self) -> NMFeagiAdvancedQuantKind {
        match self {
            Self::Standard32Bit(_) => NMFeagiAdvancedQuantKind::Standard32Bit,
        }
    }
}

/// Allows Neurogenesis of the Feagi Advanced Neuron Model from its stored data for a given quantization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum NMFeagiAdvancedNeurogenesisStorageQuanted<FIQ: FeagiIndexQuantization> {
    Standard32Bit(ProxyNMNeuroGenesisFromStorage<FIQ, NMFeagiAdvancedQuantStandard32Bit>)
}

impl<FIQ: FeagiIndexQuantization> NMFeagiAdvancedNeurogenesisStorageQuanted<FIQ> {
    /// The quantization of the storage held
    pub const fn quant_kind(&self) -> NMFeagiAdvancedQuantKind {
        match self {
            Self::Standard32Bit(_) => NMFeagiAdvancedQuantKind::Standard32Bit,
        }
    }
}

/// Allows Neurogenesis of the Feagi Advanced Neuron Model from either its stored data or by its
/// generator, for a given quantization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum NMFeagiAdvancedNeurogenesisStorageOrGeneratorQuanted<FIQ: FeagiIndexQuantization> {
    Standard32Bit(ProxyNMNeurogenesisFromStorageOrByGenerator<FIQ, NMFeagiAdvancedQuantStandard32Bit>)
}

impl<FIQ: FeagiIndexQuantization> NMFeagiAdvancedNeurogenesisStorageOrGeneratorQuanted<FIQ> {
    /// The quantization of the storage or generator held
    pub const fn quant_kind(&self) -> NMFeagiAdvancedQuantKind {
        match self {
            Self::Standard32Bit(_) => NMFeagiAdvancedQuantKind::Standard32Bit,
        }
    }
}


//endregion

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMFeagiAdvancedQuantStandard32Bit;

impl NMFeagiAdvancedTrait for NMFeagiAdvancedQuantStandard32Bit {
    type Period = f32;
}

impl NeuronModel for NMFeagiAdvancedQuantStandard32Bit {
    type NeuronLayoutTag = NeuronLayoutTagVoxel;

    type NeurogenesisScratchVarianceGeneratorType<FIQ: FeagiIndexQuantization> =
        super::neurogenesis::FeagiAdvancedScratchVarianceGenerator<FIQ, Self>;

    type NeurogenesisByGeneratorType<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorUniformDefault<FIQ, Self>;

    type CorticalDataProperties = NMFeagiAdvancedDataCorticalProperties<Self>;
    type CorticalDataInternal = NMFeagiAdvancedDataCorticalInternal<Self>;
    type CorticalDataScratch = NMFeagiAdvancedDataCorticalScratch<Self>;
    type CorticalDataVariance = NMFeagiAdvancedDataCorticalVariance<Self>;
    type NeuronDataInternal = NMFeagiAdvancedDataNeuronInternal<Self>;
    type NeuronDataScratch = NMFeagiAdvancedDataNeuronScratch<Self>;
    type NeuronDataVariance = NMFeagiAdvancedDataNeuronVariance<Self>;
    
    type QuantKind = NMFeagiAdvancedQuantKind;
    const QUANT_KIND: Self::QuantKind = NMFeagiAdvancedQuantKind::Standard32Bit;
}

impl MembranePotentialQuantization for NMFeagiAdvancedQuantStandard32Bit
{
    type MembranePotentialQuant = f32;
}

//region Model Level Conversions

impl From<NMFeagiAdvancedQuantKind> for NMKind {
    fn from(value: NMFeagiAdvancedQuantKind) -> Self {
        NMKind::FeagiAdvanced(value)
    }
}

impl TryFrom<NMKind> for NMFeagiAdvancedQuantKind {
    type Error = NeuronModelEnumConversionError<NMModelKind>;

    fn try_from(value: NMKind) -> Result<Self, Self::Error> {
        let actual = value.model_kind();
        match value {
            NMKind::FeagiAdvanced(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMModelKind::FeagiAdvanced,
                    actual,
                })
            }
        }
    }
}

impl<FIQ> From<NMFeagiAdvancedNeurogenesisGeneratorQuanted<FIQ>> for NMNeurogenesisGeneratorModeled<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: NMFeagiAdvancedNeurogenesisGeneratorQuanted<FIQ>) -> Self {
        NMNeurogenesisGeneratorModeled::FeagiAdvanced(value)
    }
}

impl<FIQ> TryFrom<NMNeurogenesisGeneratorModeled<FIQ>> for NMFeagiAdvancedNeurogenesisGeneratorQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMModelKind>;

    fn try_from(value: NMNeurogenesisGeneratorModeled<FIQ>) -> Result<Self, Self::Error> {
        let actual = value.model_kind();
        match value {
            NMNeurogenesisGeneratorModeled::FeagiAdvanced(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMModelKind::FeagiAdvanced,
                    actual,
                })
            }
        }
    }
}

impl<FIQ> From<NMFeagiAdvancedNeurogenesisStorageQuanted<FIQ>> for NMNeurogenesisStorageModeled<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: NMFeagiAdvancedNeurogenesisStorageQuanted<FIQ>) -> Self {
        NMNeurogenesisStorageModeled::FeagiAdvanced(value)
    }
}

impl<FIQ> TryFrom<NMNeurogenesisStorageModeled<FIQ>> for NMFeagiAdvancedNeurogenesisStorageQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMModelKind>;

    fn try_from(value: NMNeurogenesisStorageModeled<FIQ>) -> Result<Self, Self::Error> {
        let actual = value.model_kind();
        match value {
            NMNeurogenesisStorageModeled::FeagiAdvanced(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMModelKind::FeagiAdvanced,
                    actual,
                })
            }
        }
    }
}

impl<FIQ> From<NMFeagiAdvancedNeurogenesisStorageOrGeneratorQuanted<FIQ>> for NMNeurogenesisStorageOrGeneratorModeled<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: NMFeagiAdvancedNeurogenesisStorageOrGeneratorQuanted<FIQ>) -> Self {
        NMNeurogenesisStorageOrGeneratorModeled::FeagiAdvanced(value)
    }
}

impl<FIQ> TryFrom<NMNeurogenesisStorageOrGeneratorModeled<FIQ>> for NMFeagiAdvancedNeurogenesisStorageOrGeneratorQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMModelKind>;

    fn try_from(value: NMNeurogenesisStorageOrGeneratorModeled<FIQ>) -> Result<Self, Self::Error> {
        let actual = value.model_kind();
        match value {
            NMNeurogenesisStorageOrGeneratorModeled::FeagiAdvanced(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMModelKind::FeagiAdvanced,
                    actual,
                })
            }
        }
    }
}

//endregion
