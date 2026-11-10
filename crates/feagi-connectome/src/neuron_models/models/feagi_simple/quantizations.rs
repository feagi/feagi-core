use serde::{Deserialize, Serialize};
use crate::neuron_models::models::feagi_simple::{NMFeagiSimpleDataCorticalInternal, NMFeagiSimpleDataCorticalProperties, NMFeagiSimpleDataCorticalScratch, NMFeagiSimpleDataCorticalVariance, NMFeagiSimpleDataNeuronInternal, NMFeagiSimpleDataNeuronScratch, NMFeagiSimpleDataNeuronVariance, NMFeagiSimpleTrait};
use crate::neuron_models::models::{NMKind, NMModelKind, NMNeurogenesisGeneratorModeled, NMNeurogenesisStorageModeled, NMNeurogenesisStorageOrGeneratorModeled, NeuronModelEnumConversionError};
use crate::neuron_models::models::{ProxyNMNeuroGenesisFromGenerator, ProxyNMNeuroGenesisFromStorage, ProxyNMNeurogenesisFromStorageOrByGenerator};
use crate::neuron_models::neurogenesis_generator_uniform_defaults::NeurogenesisGeneratorUniformDefault;
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::voxel::NeuronLayoutTagVoxel;
use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome_neural::neural_basis::quantization_levels::{FeagiIndexQuantization, MembranePotentialQuantization};
//region enums

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
/// Describes the quantization of the Feagi Simple Neuron Model
pub enum NMFeagiSimpleQuantKind {
    Standard32Bit,
}

/// Allows Neurogenesis of the Feagi Simple Neuron Model by its generator for a given quantization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum NMFeagiSimpleNeurogenesisGeneratorQuanted<FIQ: FeagiIndexQuantization> {
    Standard32Bit(ProxyNMNeuroGenesisFromGenerator<FIQ, NMFeagiSimpleQuantStandard32Bit>)
}

impl<FIQ: FeagiIndexQuantization> NMFeagiSimpleNeurogenesisGeneratorQuanted<FIQ> {
    /// The quantization of the generator held
    pub const fn quant_kind(&self) -> NMFeagiSimpleQuantKind {
        match self {
            Self::Standard32Bit(_) => NMFeagiSimpleQuantKind::Standard32Bit,
        }
    }
}

/// Allows Neurogenesis of the Feagi Simple Neuron Model from its stored data for a given quantization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum NMFeagiSimpleNeurogenesisStorageQuanted<FIQ: FeagiIndexQuantization> {
    Standard32Bit(ProxyNMNeuroGenesisFromStorage<FIQ, NMFeagiSimpleQuantStandard32Bit>)
}

impl<FIQ: FeagiIndexQuantization> NMFeagiSimpleNeurogenesisStorageQuanted<FIQ> {
    /// The quantization of the storage held
    pub const fn quant_kind(&self) -> NMFeagiSimpleQuantKind {
        match self {
            Self::Standard32Bit(_) => NMFeagiSimpleQuantKind::Standard32Bit,
        }
    }
}

/// Allows Neurogenesis of the Feagi Simple Neuron Model from either its stored data or by its
/// generator, for a given quantization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum NMFeagiSimpleNeurogenesisStorageOrGeneratorQuanted<FIQ: FeagiIndexQuantization> {
    Standard32Bit(ProxyNMNeurogenesisFromStorageOrByGenerator<FIQ, NMFeagiSimpleQuantStandard32Bit>)
}

impl<FIQ: FeagiIndexQuantization> NMFeagiSimpleNeurogenesisStorageOrGeneratorQuanted<FIQ> {
    /// The quantization of the storage or generator held
    pub const fn quant_kind(&self) -> NMFeagiSimpleQuantKind {
        match self {
            Self::Standard32Bit(_) => NMFeagiSimpleQuantKind::Standard32Bit,
        }
    }
}


//endregion

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMFeagiSimpleQuantStandard32Bit;

impl NMFeagiSimpleTrait for NMFeagiSimpleQuantStandard32Bit {
    type Period = f32;
}

impl NeuronModel for NMFeagiSimpleQuantStandard32Bit {
    type NeuronLayoutTag = NeuronLayoutTagVoxel;

    type NeurogenesisScratchVarianceGeneratorType<FIQ: FeagiIndexQuantization> =
        super::neurogenesis::FeagiSimpleNeurogenesisDefaults<FIQ, Self>;

    type NeurogenesisByGeneratorType<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorUniformDefault<FIQ, Self>;

    type CorticalDataProperties = NMFeagiSimpleDataCorticalProperties<Self>;
    type CorticalDataInternal = NMFeagiSimpleDataCorticalInternal<Self>;
    type CorticalDataScratch = NMFeagiSimpleDataCorticalScratch<Self>;
    type CorticalDataVariance = NMFeagiSimpleDataCorticalVariance<Self>;
    type NeuronDataInternal = NMFeagiSimpleDataNeuronInternal<Self>;
    type NeuronDataScratch = NMFeagiSimpleDataNeuronScratch<Self>;
    type NeuronDataVariance = NMFeagiSimpleDataNeuronVariance<Self>;

    type QuantKind = NMFeagiSimpleQuantKind;
    const QUANT_KIND: Self::QuantKind = NMFeagiSimpleQuantKind::Standard32Bit;
}

impl MembranePotentialQuantization for NMFeagiSimpleQuantStandard32Bit
{
    type MembranePotentialQuant = f32;
}

//region Model Level Conversions

impl From<NMFeagiSimpleQuantKind> for NMKind {
    fn from(value: NMFeagiSimpleQuantKind) -> Self {
        NMKind::FeagiSimple(value)
    }
}

impl TryFrom<NMKind> for NMFeagiSimpleQuantKind {
    type Error = NeuronModelEnumConversionError<NMModelKind>;

    fn try_from(value: NMKind) -> Result<Self, Self::Error> {
        let actual = value.model_kind();
        match value {
            NMKind::FeagiSimple(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMModelKind::FeagiSimple,
                    actual,
                })
            }
        }
    }
}

impl<FIQ> From<NMFeagiSimpleNeurogenesisGeneratorQuanted<FIQ>> for NMNeurogenesisGeneratorModeled<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: NMFeagiSimpleNeurogenesisGeneratorQuanted<FIQ>) -> Self {
        NMNeurogenesisGeneratorModeled::FeagiSimple(value)
    }
}

impl<FIQ> TryFrom<NMNeurogenesisGeneratorModeled<FIQ>> for NMFeagiSimpleNeurogenesisGeneratorQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMModelKind>;

    fn try_from(value: NMNeurogenesisGeneratorModeled<FIQ>) -> Result<Self, Self::Error> {
        let actual = value.model_kind();
        match value {
            NMNeurogenesisGeneratorModeled::FeagiSimple(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMModelKind::FeagiSimple,
                    actual,
                })
            }
        }
    }
}

impl<FIQ> From<NMFeagiSimpleNeurogenesisStorageQuanted<FIQ>> for NMNeurogenesisStorageModeled<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: NMFeagiSimpleNeurogenesisStorageQuanted<FIQ>) -> Self {
        NMNeurogenesisStorageModeled::FeagiSimple(value)
    }
}

impl<FIQ> TryFrom<NMNeurogenesisStorageModeled<FIQ>> for NMFeagiSimpleNeurogenesisStorageQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMModelKind>;

    fn try_from(value: NMNeurogenesisStorageModeled<FIQ>) -> Result<Self, Self::Error> {
        let actual = value.model_kind();
        match value {
            NMNeurogenesisStorageModeled::FeagiSimple(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMModelKind::FeagiSimple,
                    actual,
                })
            }
        }
    }
}

impl<FIQ> From<NMFeagiSimpleNeurogenesisStorageOrGeneratorQuanted<FIQ>> for NMNeurogenesisStorageOrGeneratorModeled<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: NMFeagiSimpleNeurogenesisStorageOrGeneratorQuanted<FIQ>) -> Self {
        NMNeurogenesisStorageOrGeneratorModeled::FeagiSimple(value)
    }
}

impl<FIQ> TryFrom<NMNeurogenesisStorageOrGeneratorModeled<FIQ>> for NMFeagiSimpleNeurogenesisStorageOrGeneratorQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMModelKind>;

    fn try_from(value: NMNeurogenesisStorageOrGeneratorModeled<FIQ>) -> Result<Self, Self::Error> {
        let actual = value.model_kind();
        match value {
            NMNeurogenesisStorageOrGeneratorModeled::FeagiSimple(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMModelKind::FeagiSimple,
                    actual,
                })
            }
        }
    }
}

//endregion
