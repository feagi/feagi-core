use serde::{Deserialize, Serialize};
use crate::neuron_models::models::feagi_simple::{NMFeagiSimpleDataCorticalInternal, NMFeagiSimpleDataCorticalProperties, NMFeagiSimpleDataCorticalScratch, NMFeagiSimpleDataCorticalVariance, NMFeagiSimpleDataNeuronInternal, NMFeagiSimpleDataNeuronScratch, NMFeagiSimpleDataNeuronVariance, NMFeagiSimpleTrait};
use crate::neuron_models::models::{NMNeurogenesisGeneratorModeled, NMNeurogenesisGeneratorPrecursorModeled, NMNeurogenesisStorageModeled, NMNeurogenesisStorageOrGeneratorModeled, NeuronModelEnumConversionError};
use crate::neuron_models::models::{NeurogenesisByGeneratorPrecursorTypeOf, NeurogenesisByGeneratorTypeOf, NeurogenesisFromStorageOf, NeurogenesisFromStorageOrByGeneratorOf};
use crate::neuron_models::neurogenesis_generator_uniform_defaults::{NeurogenesisGeneratorPrecursorUniformDefault, NeurogenesisGeneratorUniformDefault};
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::voxel::NeuronLayoutTagVoxel;
use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome_neural::neural_basis::quantization_levels::{FeagiIndexQuantization, MembranePotentialQuantization};
use crate::feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::from_storage::NeurogenesisStorage;
use crate::neuron_models::models::kind::{NMKind, NMModelKind};

//region enums

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
/// Describes the quantization of the Feagi Simple Neuron Model
pub enum NMFeagiSimpleQuantKind {
    Standard32Bit,
}

impl NMFeagiSimpleQuantKind {
    pub fn from_nm_quant<NM: NMFeagiSimpleTrait>() -> NMFeagiSimpleQuantKind {NM::QUANT_KIND}
}

pub enum NMFeagiSimpleQuantedKind {
    Standard32Bit(NMFeagiSimpleQuantStandard32Bit)
}

impl NMFeagiSimpleQuantedKind {
    pub fn quant_kind(&self) -> NMFeagiSimpleQuantKind {
        match &self { 
            NMFeagiSimpleQuantedKind::Standard32Bit(_) => NMFeagiSimpleQuantKind::Standard32Bit,
        }
        
    }
    
    pub fn from_nm_quant<NM: NMFeagiSimpleTrait>() -> NMFeagiSimpleQuantedKind {
        match NM::QUANT_KIND {
            NMFeagiSimpleQuantKind::Standard32Bit => NMFeagiSimpleQuantedKind::Standard32Bit(NMFeagiSimpleQuantStandard32Bit)
        }
    }
} 


//region Generators

/// The data needed to create the generator of the Feagi Simple Neuron Model, for a given quantization
#[derive(Debug, Clone)]
pub enum NMFeagiSimpleNeurogenesisGeneratorPrecursorQuanted<FIQ: FeagiIndexQuantization> {
    Standard32Bit(NeurogenesisByGeneratorPrecursorTypeOf<FIQ, NMFeagiSimpleQuantStandard32Bit>)
}

impl<FIQ: FeagiIndexQuantization> NMFeagiSimpleNeurogenesisGeneratorPrecursorQuanted<FIQ> {
    /// The quantization of the precursor held
    pub const fn quant_kind(&self) -> NMFeagiSimpleQuantKind {
        match self {
            Self::Standard32Bit(_) => NMFeagiSimpleQuantKind::Standard32Bit,
        }
    }
}

/// Allows Neurogenesis of the Feagi Simple Neuron Model by its generator for a given quantization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum NMFeagiSimpleNeurogenesisGeneratorQuanted<FIQ: FeagiIndexQuantization> {
    Standard32Bit(NeurogenesisByGeneratorTypeOf<FIQ, NMFeagiSimpleQuantStandard32Bit>)
}

impl<FIQ: FeagiIndexQuantization> NMFeagiSimpleNeurogenesisGeneratorQuanted<FIQ> {
    /// The quantization of the generator held
    pub const fn quant_kind(&self) -> NMFeagiSimpleQuantKind {
        match self {
            Self::Standard32Bit(_) => NMFeagiSimpleQuantKind::Standard32Bit,
        }
    }
    
    // NOTE: do not offer `from_nm_quant` since the NM will not have it, thats up to the CT
    // to provide an implementation
}

/// Allows Neurogenesis of the Feagi Simple Neuron Model from its stored data for a given quantization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum NMFeagiSimpleNeurogenesisStorageQuanted<FIQ: FeagiIndexQuantization> {
    Standard32Bit(NeurogenesisFromStorageOf<FIQ, NMFeagiSimpleQuantStandard32Bit>)
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
    Standard32Bit(NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMFeagiSimpleQuantStandard32Bit>)
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


//endregion

//region Quant Levels

//region Standard32Bit

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMFeagiSimpleQuantStandard32Bit;

impl NMFeagiSimpleTrait for NMFeagiSimpleQuantStandard32Bit {
    type Period = f32;
}

impl NeuronModel for NMFeagiSimpleQuantStandard32Bit {
    type NeuronLayoutTag = NeuronLayoutTagVoxel;

    type NeurogenesisScratchVarianceGeneratorType<FIQ: FeagiIndexQuantization> =
        super::neurogenesis::NMFeagiSimpleScratchVarianceGenerator<FIQ, Self>;

    type NeurogenesisGenerator<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorUniformDefault<FIQ, Self>;
    type NeurogenesisGeneratorPrecursor<FIQ: FeagiIndexQuantization> = 
        NeurogenesisGeneratorPrecursorUniformDefault<FIQ, Self>;

    type CorticalDataProperties = NMFeagiSimpleDataCorticalProperties<Self>;
    type CorticalDataInternal = NMFeagiSimpleDataCorticalInternal<Self>;
    type CorticalDataScratch = NMFeagiSimpleDataCorticalScratch<Self>;
    type CorticalDataVariance = NMFeagiSimpleDataCorticalVariance<Self>;
    type NeuronDataInternal = NMFeagiSimpleDataNeuronInternal<Self>;
    type NeuronDataScratch = NMFeagiSimpleDataNeuronScratch<Self>;
    type NeuronDataVariance = NMFeagiSimpleDataNeuronVariance<Self>;

    type QuantKind = NMFeagiSimpleQuantKind;
    type QuantedKind = NMFeagiSimpleQuantedKind;

    const QUANT_KIND: Self::QuantKind = NMFeagiSimpleQuantKind::Standard32Bit;
    const QUANTED_KIND: Self::QuantedKind = NMFeagiSimpleQuantedKind::Standard32Bit(Self);
}

impl MembranePotentialQuantization for NMFeagiSimpleQuantStandard32Bit {
    type MembranePotentialQuant = f32;
}

impl From<NMFeagiSimpleQuantStandard32Bit> for NMFeagiSimpleQuantedKind {
    fn from(value: NMFeagiSimpleQuantStandard32Bit) -> Self {
        NMFeagiSimpleQuantedKind::Standard32Bit(value)
    }
}

impl TryFrom<NMFeagiSimpleQuantedKind> for NMFeagiSimpleQuantStandard32Bit {
    type Error = NeuronModelEnumConversionError<NMFeagiSimpleQuantKind>;

    fn try_from(value: NMFeagiSimpleQuantedKind) -> Result<Self, Self::Error> {
        let actual = value.quant_kind();
        match value {
            NMFeagiSimpleQuantedKind::Standard32Bit(s) => {Ok(s)},
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMFeagiSimpleQuantKind::Standard32Bit,
                    actual,
                })
            }
        }
    }
}

//endregion

//endregion

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

impl<FIQ> From<NMFeagiSimpleNeurogenesisGeneratorPrecursorQuanted<FIQ>> for NMNeurogenesisGeneratorPrecursorModeled<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: NMFeagiSimpleNeurogenesisGeneratorPrecursorQuanted<FIQ>) -> Self {
        NMNeurogenesisGeneratorPrecursorModeled::FeagiSimple(value)
    }
}

impl<FIQ> TryFrom<NMNeurogenesisGeneratorPrecursorModeled<FIQ>> for NMFeagiSimpleNeurogenesisGeneratorPrecursorQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMModelKind>;

    fn try_from(value: NMNeurogenesisGeneratorPrecursorModeled<FIQ>) -> Result<Self, Self::Error> {
        let actual = value.model_kind();
        match value {
            NMNeurogenesisGeneratorPrecursorModeled::FeagiSimple(v) => {
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
