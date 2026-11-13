use serde::{Deserialize, Serialize};
use crate::neuron_models::models::feagi_advanced::{NMFeagiAdvancedDataCorticalInternal, NMFeagiAdvancedDataCorticalProperties, NMFeagiAdvancedDataCorticalScratch, NMFeagiAdvancedDataCorticalVariance, NMFeagiAdvancedDataNeuronInternal, NMFeagiAdvancedDataNeuronScratch, NMFeagiAdvancedDataNeuronVariance, NMFeagiAdvancedTrait};
use crate::neuron_models::models::kind::{NMKind, NMQuantedKind};
use crate::neuron_models::models::{NMNeurogenesisGeneratorModeled, NMNeurogenesisGeneratorPrecursorModeled, NMNeurogenesisStorageModeled, NMNeurogenesisStorageOrGeneratorModeled, NeuronModelEnumConversionError};
use crate::neuron_models::models::{NeurogenesisByGeneratorPrecursorTypeOf, NeurogenesisByGeneratorTypeOf, NeurogenesisFromStorageOf, NeurogenesisFromStorageOrByGeneratorOf};
use crate::neuron_models::neurogenesis_generator_uniform_defaults::{NeurogenesisGeneratorPrecursorUniformDefault, NeurogenesisGeneratorUniformDefault};
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::voxel::NeuronLayoutTagVoxel;
use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome_neural::neural_basis::quantization_levels::{FeagiIndexQuantization, MembranePotentialQuantization};


//region enums

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
/// Describes the quantization of the Feagi Advanced Neuron Model
pub enum NMFeagiAdvancedQuantKind {
    Standard32Bit,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum NMFeagiAdvancedQuantedKind {
    Standard32Bit(NMFeagiAdvancedQuantStandard32Bit)
}

impl NMFeagiAdvancedQuantedKind {
    pub fn quant_kind(&self) -> NMFeagiAdvancedQuantKind {
        match &self {
            NMFeagiAdvancedQuantedKind::Standard32Bit(_) => NMFeagiAdvancedQuantKind::Standard32Bit,
        }
    }

    pub fn from_nm_quant<NM: NMFeagiAdvancedTrait>() -> NMFeagiAdvancedQuantedKind {
        match NM::QUANT_KIND {
            NMFeagiAdvancedQuantKind::Standard32Bit => NMFeagiAdvancedQuantedKind::Standard32Bit(NMFeagiAdvancedQuantStandard32Bit),
        }
    }
}


//region Generators

/// The data needed to create the generator of the Feagi Advanced Neuron Model, for a given quantization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum NMFeagiAdvancedNeurogenesisGeneratorPrecursorQuanted<FIQ: FeagiIndexQuantization> {
    Standard32Bit(NeurogenesisByGeneratorPrecursorTypeOf<FIQ, NMFeagiAdvancedQuantStandard32Bit>)
}

impl<FIQ: FeagiIndexQuantization> NMFeagiAdvancedNeurogenesisGeneratorPrecursorQuanted<FIQ> {
    /// The quantization of the precursor held
    pub const fn quant_kind(&self) -> NMFeagiAdvancedQuantKind {
        match self {
            Self::Standard32Bit(_) => NMFeagiAdvancedQuantKind::Standard32Bit,
        }
    }
}

/// Allows Neurogenesis of the Feagi Advanced Neuron Model by its generator for a given quantization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum NMFeagiAdvancedNeurogenesisGeneratorQuanted<FIQ: FeagiIndexQuantization> {
    Standard32Bit(NeurogenesisByGeneratorTypeOf<FIQ, NMFeagiAdvancedQuantStandard32Bit>)
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
    Standard32Bit(NeurogenesisFromStorageOf<FIQ, NMFeagiAdvancedQuantStandard32Bit>)
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
    Standard32Bit(NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMFeagiAdvancedQuantStandard32Bit>)
}

impl<FIQ: FeagiIndexQuantization> NMFeagiAdvancedNeurogenesisStorageOrGeneratorQuanted<FIQ> {
    /// The quantization of the storage or generator held
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
pub enum NMFeagiAdvancedNeurogenesisStorageOrGeneratorPrecursorQuanted<FIQ: FeagiIndexQuantization> {
    Standard32Bit(NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMFeagiAdvancedQuantStandard32Bit>)
}

impl<FIQ: FeagiIndexQuantization> NMFeagiAdvancedNeurogenesisStorageOrGeneratorPrecursorQuanted<FIQ> {
    /// The quantization of the storage or generator held
    pub const fn quant_kind(&self) -> NMFeagiAdvancedQuantKind {
        match self {
            Self::Standard32Bit(_) => NMFeagiAdvancedQuantKind::Standard32Bit,
        }
    }
}

//endregion

//endregion

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMFeagiAdvancedQuantStandard32Bit;

impl NMFeagiAdvancedTrait for NMFeagiAdvancedQuantStandard32Bit {
    type Period = f32;
}

impl NeuronModel for NMFeagiAdvancedQuantStandard32Bit {
    type NeuronLayoutTag = NeuronLayoutTagVoxel;

    type NeurogenesisScratchVarianceDefaults<FIQ: FeagiIndexQuantization> =
        super::neurogenesis::FeagiAdvancedScratchVarianceGenerator<FIQ, Self>;

    type NeurogenesisGenerator<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorUniformDefault<FIQ, Self>;
    type NeurogenesisGeneratorPrecursor<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorPrecursorUniformDefault<FIQ, Self>;

    type CorticalDataProperties = NMFeagiAdvancedDataCorticalProperties<Self>;
    type CorticalDataInternal = NMFeagiAdvancedDataCorticalInternal<Self>;
    type CorticalDataScratch = NMFeagiAdvancedDataCorticalScratch<Self>;
    type CorticalDataVariance = NMFeagiAdvancedDataCorticalVariance<Self>;
    type NeuronDataInternal = NMFeagiAdvancedDataNeuronInternal<Self>;
    type NeuronDataScratch = NMFeagiAdvancedDataNeuronScratch<Self>;
    type NeuronDataVariance = NMFeagiAdvancedDataNeuronVariance<Self>;
    
    type QuantKind = NMFeagiAdvancedQuantKind;
    type QuantedKind = NMFeagiAdvancedQuantedKind;
    const QUANT_KIND: Self::QuantKind = NMFeagiAdvancedQuantKind::Standard32Bit;
    const QUANTED_KIND: Self::QuantedKind = NMFeagiAdvancedQuantedKind::Standard32Bit(Self);
}

impl MembranePotentialQuantization for NMFeagiAdvancedQuantStandard32Bit
{
    type MembranePotentialQuant = f32;
}

impl From<NMFeagiAdvancedQuantStandard32Bit> for NMFeagiAdvancedQuantedKind {
    fn from(value: NMFeagiAdvancedQuantStandard32Bit) -> Self {
        NMFeagiAdvancedQuantedKind::Standard32Bit(value)
    }
}

impl TryFrom<NMFeagiAdvancedQuantedKind> for NMFeagiAdvancedQuantStandard32Bit {
    type Error = NeuronModelEnumConversionError<NMFeagiAdvancedQuantKind>;

    fn try_from(value: NMFeagiAdvancedQuantedKind) -> Result<Self, Self::Error> {
        let actual = value.quant_kind();
        match value {
            NMFeagiAdvancedQuantedKind::Standard32Bit(s) => {Ok(s)},
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMFeagiAdvancedQuantKind::Standard32Bit,
                    actual,
                })
            }
        }
    }
}

//region Model Level Conversions

impl From<NMFeagiAdvancedQuantKind> for NMKind {
    fn from(value: NMFeagiAdvancedQuantKind) -> Self {
        NMKind::FeagiAdvanced(value)
    }
}

impl TryFrom<NMKind> for NMFeagiAdvancedQuantKind {
    type Error = NeuronModelEnumConversionError<NMKind>;

    fn try_from(value: NMKind) -> Result<Self, Self::Error> {
        match value {
            NMKind::FeagiAdvanced(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleModel)
            }
        }
    }
}

impl From<NMFeagiAdvancedQuantedKind> for NMQuantedKind {
    fn from(value: NMFeagiAdvancedQuantedKind) -> Self {
        NMQuantedKind::FeagiAdvanced(value)
    }
}

impl TryFrom<NMQuantedKind> for NMFeagiAdvancedQuantedKind {
    type Error = NeuronModelEnumConversionError<NMKind>;

    fn try_from(value: NMQuantedKind) -> Result<Self, Self::Error> {
        match value {
            NMQuantedKind::FeagiAdvanced(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleModel)
            }
        }
    }
}

impl<FIQ> From<NMFeagiAdvancedNeurogenesisGeneratorPrecursorQuanted<FIQ>> for NMNeurogenesisGeneratorPrecursorModeled<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: NMFeagiAdvancedNeurogenesisGeneratorPrecursorQuanted<FIQ>) -> Self {
        NMNeurogenesisGeneratorPrecursorModeled::FeagiAdvanced(value)
    }
}

impl<FIQ> TryFrom<NMNeurogenesisGeneratorPrecursorModeled<FIQ>> for NMFeagiAdvancedNeurogenesisGeneratorPrecursorQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMKind>;

    fn try_from(value: NMNeurogenesisGeneratorPrecursorModeled<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMNeurogenesisGeneratorPrecursorModeled::FeagiAdvanced(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleModel)
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
    type Error = NeuronModelEnumConversionError<NMKind>;

    fn try_from(value: NMNeurogenesisGeneratorModeled<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMNeurogenesisGeneratorModeled::FeagiAdvanced(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleModel)
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
    type Error = NeuronModelEnumConversionError<NMKind>;

    fn try_from(value: NMNeurogenesisStorageModeled<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMNeurogenesisStorageModeled::FeagiAdvanced(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleModel)
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
    type Error = NeuronModelEnumConversionError<NMKind>;

    fn try_from(value: NMNeurogenesisStorageOrGeneratorModeled<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMNeurogenesisStorageOrGeneratorModeled::FeagiAdvanced(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleModel)
            }
        }
    }
}

//endregion
