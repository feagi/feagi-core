use serde::{Deserialize, Serialize};
use crate::neuron_models::models::memory_fire_hashing::{NMMemoryFireHashingDataCorticalInternal, NMMemoryFireHashingDataCorticalProperties, NMMemoryFireHashingDataCorticalScratch, NMMemoryFireHashingDataCorticalVariance, NMMemoryFireHashingDataNeuronInternal, NMMemoryFireHashingDataNeuronScratch, NMMemoryFireHashingDataNeuronVariance, NMMemoryFireHashingTrait};
use crate::neuron_models::models::kind::{NMKind, NMQuantedKind};
use crate::neuron_models::models::{NMNeurogenesisGeneratorModeled, NMNeurogenesisGeneratorPrecursorModeled, NMNeurogenesisStorageModeled, NMNeurogenesisStorageOrGeneratorModeled, NeuronModelEnumConversionError};
use crate::neuron_models::models::{NeurogenesisByGeneratorPrecursorTypeOf, NeurogenesisByGeneratorTypeOf, NeurogenesisFromStorageOf, NeurogenesisFromStorageOrByGeneratorOf};
use crate::neuron_models::neurogenesis_generator_uniform_defaults::{NeurogenesisGeneratorPrecursorUniformDefault, NeurogenesisGeneratorUniformDefault};
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::single_stepped_dynamic::NeuronLayoutTagSingleSteppedDynamic;
use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome_neural::neural_basis::quantization_levels::{FeagiIndexQuantization, MembranePotentialQuantization};
//region enums

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
/// Describes the quantization of the Memory Fire Hashing Neuron Model
pub enum NMMemoryFireHashingQuantKind {
    Standard32Bit,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum NMMemoryFireHashingQuantedKind {
    Standard32Bit(NMMemoryFireHashingQuantStandard32Bit)
}

impl NMMemoryFireHashingQuantedKind {
    pub fn quant_kind(&self) -> NMMemoryFireHashingQuantKind {
        match &self {
            NMMemoryFireHashingQuantedKind::Standard32Bit(_) => NMMemoryFireHashingQuantKind::Standard32Bit,
        }
    }

    pub fn from_nm_quant<NM: NMMemoryFireHashingTrait>() -> NMMemoryFireHashingQuantedKind {
        match NM::QUANT_KIND {
            NMMemoryFireHashingQuantKind::Standard32Bit => NMMemoryFireHashingQuantedKind::Standard32Bit(NMMemoryFireHashingQuantStandard32Bit),
        }
    }
}


//region Generators

/// The data needed to create the generator of the Memory Fire Hashing Neuron Model, for a given quantization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum NMMemoryFireHashingQuantedNeurogenesisGeneratorPrecursor<FIQ: FeagiIndexQuantization> {
    Standard32Bit(NeurogenesisByGeneratorPrecursorTypeOf<FIQ, NMMemoryFireHashingQuantStandard32Bit>)
}

impl<FIQ: FeagiIndexQuantization> NMMemoryFireHashingQuantedNeurogenesisGeneratorPrecursor<FIQ> {
    /// The quantization of the precursor held
    pub const fn quant_kind(&self) -> NMMemoryFireHashingQuantKind {
        match self {
            Self::Standard32Bit(_) => NMMemoryFireHashingQuantKind::Standard32Bit,
        }
    }
}

/// Allows Neurogenesis of the Memory Fire Hashing Neuron Model by its generator for a given quantization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum NMMemoryFireHashingNeurogenesisGeneratorQuanted<FIQ: FeagiIndexQuantization> {
    Standard32Bit(NeurogenesisByGeneratorTypeOf<FIQ, NMMemoryFireHashingQuantStandard32Bit>)
}

impl<FIQ: FeagiIndexQuantization> NMMemoryFireHashingNeurogenesisGeneratorQuanted<FIQ> {
    /// The quantization of the generator held
    pub const fn quant_kind(&self) -> NMMemoryFireHashingQuantKind {
        match self {
            Self::Standard32Bit(_) => NMMemoryFireHashingQuantKind::Standard32Bit,
        }
    }
}

/// Allows Neurogenesis of the Memory Fire Hashing Neuron Model from its stored data for a given quantization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum NMMemoryFireHashingNeurogenesisStorageQuanted<FIQ: FeagiIndexQuantization> {
    Standard32Bit(NeurogenesisFromStorageOf<FIQ, NMMemoryFireHashingQuantStandard32Bit>)
}

impl<FIQ: FeagiIndexQuantization> NMMemoryFireHashingNeurogenesisStorageQuanted<FIQ> {
    /// The quantization of the storage held
    pub const fn quant_kind(&self) -> NMMemoryFireHashingQuantKind {
        match self {
            Self::Standard32Bit(_) => NMMemoryFireHashingQuantKind::Standard32Bit,
        }
    }
}

/// Allows Neurogenesis of the Memory Fire Hashing Neuron Model from either its stored data or by its
/// generator, for a given quantization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum NMMemoryFireHashingNeurogenesisStorageOrGeneratorQuanted<FIQ: FeagiIndexQuantization> {
    Standard32Bit(NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMMemoryFireHashingQuantStandard32Bit>)
}

impl<FIQ: FeagiIndexQuantization> NMMemoryFireHashingNeurogenesisStorageOrGeneratorQuanted<FIQ> {
    /// The quantization of the storage or generator held
    pub const fn quant_kind(&self) -> NMMemoryFireHashingQuantKind {
        match self {
            Self::Standard32Bit(_) => NMMemoryFireHashingQuantKind::Standard32Bit,
        }
    }
}


//endregion

//endregion

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMMemoryFireHashingQuantStandard32Bit;

impl NMMemoryFireHashingTrait for NMMemoryFireHashingQuantStandard32Bit {
    type Period = f32;
}

impl NeuronModel for NMMemoryFireHashingQuantStandard32Bit {
    type NeuronLayoutTag = NeuronLayoutTagSingleSteppedDynamic;

    type NeurogenesisScratchVarianceGeneratorType<FIQ: FeagiIndexQuantization> =
        super::neurogenesis::MemoryFireHashingNeurogenesisDefaults<FIQ, Self>;

    type NeurogenesisGenerator<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorUniformDefault<FIQ, Self>;
    type NeurogenesisGeneratorPrecursor<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorPrecursorUniformDefault<FIQ, Self>;

    type CorticalDataProperties = NMMemoryFireHashingDataCorticalProperties<Self>;
    type CorticalDataInternal = NMMemoryFireHashingDataCorticalInternal<Self>;
    type CorticalDataScratch = NMMemoryFireHashingDataCorticalScratch<Self>;
    type CorticalDataVariance = NMMemoryFireHashingDataCorticalVariance<Self>;
    type NeuronDataInternal = NMMemoryFireHashingDataNeuronInternal<Self>;
    type NeuronDataScratch = NMMemoryFireHashingDataNeuronScratch<Self>;
    type NeuronDataVariance = NMMemoryFireHashingDataNeuronVariance<Self>;

    type QuantKind = NMMemoryFireHashingQuantKind;
    type QuantedKind = NMMemoryFireHashingQuantedKind;
    const QUANT_KIND: Self::QuantKind = NMMemoryFireHashingQuantKind::Standard32Bit;
    const QUANTED_KIND: Self::QuantedKind = NMMemoryFireHashingQuantedKind::Standard32Bit(Self);
}

impl MembranePotentialQuantization for NMMemoryFireHashingQuantStandard32Bit
{
    type MembranePotentialQuant = f32;
}

impl From<NMMemoryFireHashingQuantStandard32Bit> for NMMemoryFireHashingQuantedKind {
    fn from(value: NMMemoryFireHashingQuantStandard32Bit) -> Self {
        NMMemoryFireHashingQuantedKind::Standard32Bit(value)
    }
}

impl TryFrom<NMMemoryFireHashingQuantedKind> for NMMemoryFireHashingQuantStandard32Bit {
    type Error = NeuronModelEnumConversionError<NMMemoryFireHashingQuantKind>;

    fn try_from(value: NMMemoryFireHashingQuantedKind) -> Result<Self, Self::Error> {
        let actual = value.quant_kind();
        match value {
            NMMemoryFireHashingQuantedKind::Standard32Bit(s) => {Ok(s)},
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMMemoryFireHashingQuantKind::Standard32Bit,
                    actual,
                })
            }
        }
    }
}

//region Model Level Conversions

impl From<NMMemoryFireHashingQuantKind> for NMKind {
    fn from(value: NMMemoryFireHashingQuantKind) -> Self {
        NMKind::MemoryFireHashing(value)
    }
}

impl TryFrom<NMKind> for NMMemoryFireHashingQuantKind {
    type Error = NeuronModelEnumConversionError<NMKind>;

    fn try_from(value: NMKind) -> Result<Self, Self::Error> {
        match value {
            NMKind::MemoryFireHashing(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleModel)
            }
        }
    }
}

impl From<NMMemoryFireHashingQuantedKind> for NMQuantedKind {
    fn from(value: NMMemoryFireHashingQuantedKind) -> Self {
        NMQuantedKind::MemoryFireHashing(value)
    }
}

impl TryFrom<NMQuantedKind> for NMMemoryFireHashingQuantedKind {
    type Error = NeuronModelEnumConversionError<NMKind>;

    fn try_from(value: NMQuantedKind) -> Result<Self, Self::Error> {
        match value {
            NMQuantedKind::MemoryFireHashing(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleModel)
            }
        }
    }
}

impl<FIQ> From<NMMemoryFireHashingQuantedNeurogenesisGeneratorPrecursor<FIQ>> for NMNeurogenesisGeneratorPrecursorModeled<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: NMMemoryFireHashingQuantedNeurogenesisGeneratorPrecursor<FIQ>) -> Self {
        NMNeurogenesisGeneratorPrecursorModeled::MemoryFireHashing(value)
    }
}

impl<FIQ> TryFrom<NMNeurogenesisGeneratorPrecursorModeled<FIQ>> for NMMemoryFireHashingQuantedNeurogenesisGeneratorPrecursor<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMKind>;

    fn try_from(value: NMNeurogenesisGeneratorPrecursorModeled<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMNeurogenesisGeneratorPrecursorModeled::MemoryFireHashing(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleModel)
            }
        }
    }
}

impl<FIQ> From<NMMemoryFireHashingNeurogenesisGeneratorQuanted<FIQ>> for NMNeurogenesisGeneratorModeled<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: NMMemoryFireHashingNeurogenesisGeneratorQuanted<FIQ>) -> Self {
        NMNeurogenesisGeneratorModeled::MemoryFireHashing(value)
    }
}

impl<FIQ> TryFrom<NMNeurogenesisGeneratorModeled<FIQ>> for NMMemoryFireHashingNeurogenesisGeneratorQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMKind>;

    fn try_from(value: NMNeurogenesisGeneratorModeled<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMNeurogenesisGeneratorModeled::MemoryFireHashing(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleModel)
            }
        }
    }
}

impl<FIQ> From<NMMemoryFireHashingNeurogenesisStorageQuanted<FIQ>> for NMNeurogenesisStorageModeled<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: NMMemoryFireHashingNeurogenesisStorageQuanted<FIQ>) -> Self {
        NMNeurogenesisStorageModeled::MemoryFireHashing(value)
    }
}

impl<FIQ> TryFrom<NMNeurogenesisStorageModeled<FIQ>> for NMMemoryFireHashingNeurogenesisStorageQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMKind>;

    fn try_from(value: NMNeurogenesisStorageModeled<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMNeurogenesisStorageModeled::MemoryFireHashing(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleModel)
            }
        }
    }
}

impl<FIQ> From<NMMemoryFireHashingNeurogenesisStorageOrGeneratorQuanted<FIQ>> for NMNeurogenesisStorageOrGeneratorModeled<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: NMMemoryFireHashingNeurogenesisStorageOrGeneratorQuanted<FIQ>) -> Self {
        NMNeurogenesisStorageOrGeneratorModeled::MemoryFireHashing(value)
    }
}

impl<FIQ> TryFrom<NMNeurogenesisStorageOrGeneratorModeled<FIQ>> for NMMemoryFireHashingNeurogenesisStorageOrGeneratorQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMKind>;

    fn try_from(value: NMNeurogenesisStorageOrGeneratorModeled<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMNeurogenesisStorageOrGeneratorModeled::MemoryFireHashing(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleModel)
            }
        }
    }
}

//endregion
