use crate::neuron_models::models::feagi_simple::{NMFeagiSimpleDataCorticalInternal, NMFeagiSimpleDataCorticalProperties, NMFeagiSimpleDataCorticalScratch, NMFeagiSimpleDataCorticalVariance, NMFeagiSimpleDataNeuronInternal, NMFeagiSimpleDataNeuronScratch, NMFeagiSimpleDataNeuronVariance, NMFeagiSimpleTrait};
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::voxel::NeuronLayoutTagVoxel;
use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome_neural::neural_basis::quantization_levels::{FeagiIndexQuantization, MembranePotentialQuantization};
use crate::neuron_models::models::kind::{NMKind, NMQuantedKind};
use crate::neuron_models::models::{NMNeurogenesisGeneratorModeled, NMNeurogenesisValidatedGeneratorModeled, NeuronModelEnumConversionError, NeurogenesisByGeneratorTypeOf};
#[cfg(feature = "alloc")]
use crate::neuron_models::models::{NMNeurogenesisStorageModeled, NMNeurogenesisStorageOrGeneratorModeled, NMNeurogenesisValidatedStorageModeled, NMNeurogenesisValidatedStorageOrGeneratorModeled, NeurogenesisFromStorageOf, NeurogenesisFromStorageOrByGeneratorOf};
use crate::neuron_models::neurogenesis_generator_uniform_defaults::NeurogenesisGeneratorUniformDefault;
use feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::validation::ValidatedNeurogenesisGenerator;
#[cfg(feature = "alloc")]
use feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::validation::{ValidatedNeurogenesisStorage, ValidatedNeurogenesisStorageOrGenerator};

//region enums

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
/// Describes the quantization of the Feagi Simple Neuron Model
pub enum NMFeagiSimpleQuantKind {
    Standard32Bit,
}

impl NMFeagiSimpleQuantKind {
    pub fn from_nm_quant<NM: NMFeagiSimpleTrait>() -> NMFeagiSimpleQuantKind {NM::QUANT_KIND}
}

#[derive(Clone, Copy, PartialEq, Debug)]
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


//region Neurogenesis

/// The neurogenesis generator of the Feagi Simple Neuron Model, for a given quantization
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(bound = "")]
pub enum NMFeagiSimpleNeurogenesisGeneratorQuanted<FIQ: FeagiIndexQuantization> {
    Standard32Bit(NeurogenesisByGeneratorTypeOf<FIQ, NMFeagiSimpleQuantStandard32Bit>),
}

impl<FIQ: FeagiIndexQuantization> NMFeagiSimpleNeurogenesisGeneratorQuanted<FIQ> {
    /// The quantization of the data held
    pub const fn quant_kind(&self) -> NMFeagiSimpleQuantKind {
        match self {
            Self::Standard32Bit(_) => NMFeagiSimpleQuantKind::Standard32Bit,
        }
    }
}

/// The stored neuron data of the Feagi Simple Neuron Model, for a given quantization
#[cfg(feature = "alloc")]
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(bound = "")]
pub enum NMFeagiSimpleNeurogenesisStorageQuanted<FIQ: FeagiIndexQuantization> {
    Standard32Bit(NeurogenesisFromStorageOf<FIQ, NMFeagiSimpleQuantStandard32Bit>),
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> NMFeagiSimpleNeurogenesisStorageQuanted<FIQ> {
    /// The quantization of the data held
    pub const fn quant_kind(&self) -> NMFeagiSimpleQuantKind {
        match self {
            Self::Standard32Bit(_) => NMFeagiSimpleQuantKind::Standard32Bit,
        }
    }
}

/// The stored neuron data or the generator of the Feagi Simple Neuron Model, for a given quantization
#[cfg(feature = "alloc")]
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(bound = "")]
pub enum NMFeagiSimpleNeurogenesisStorageOrGeneratorQuanted<FIQ: FeagiIndexQuantization> {
    Standard32Bit(NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMFeagiSimpleQuantStandard32Bit>),
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> NMFeagiSimpleNeurogenesisStorageOrGeneratorQuanted<FIQ> {
    /// The quantization of the data held
    pub const fn quant_kind(&self) -> NMFeagiSimpleQuantKind {
        match self {
            Self::Standard32Bit(_) => NMFeagiSimpleQuantKind::Standard32Bit,
        }
    }
}

/// The validated neurogenesis generator of the Feagi Simple Neuron Model, for a given quantization
#[derive(Debug, Clone)]
pub enum NMFeagiSimpleNeurogenesisValidatedGeneratorQuanted<FIQ: FeagiIndexQuantization> {
    Standard32Bit(ValidatedNeurogenesisGenerator<FIQ, NMFeagiSimpleQuantStandard32Bit>),
}

impl<FIQ: FeagiIndexQuantization> NMFeagiSimpleNeurogenesisValidatedGeneratorQuanted<FIQ> {
    /// The quantization of the data held
    pub const fn quant_kind(&self) -> NMFeagiSimpleQuantKind {
        match self {
            Self::Standard32Bit(_) => NMFeagiSimpleQuantKind::Standard32Bit,
        }
    }
}

/// The validated stored neuron data of the Feagi Simple Neuron Model, for a given quantization
#[cfg(feature = "alloc")]
#[derive(Debug, Clone)]
pub enum NMFeagiSimpleNeurogenesisValidatedStorageQuanted<FIQ: FeagiIndexQuantization> {
    Standard32Bit(ValidatedNeurogenesisStorage<FIQ, NMFeagiSimpleQuantStandard32Bit>),
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> NMFeagiSimpleNeurogenesisValidatedStorageQuanted<FIQ> {
    /// The quantization of the data held
    pub const fn quant_kind(&self) -> NMFeagiSimpleQuantKind {
        match self {
            Self::Standard32Bit(_) => NMFeagiSimpleQuantKind::Standard32Bit,
        }
    }
}

/// The validated stored neuron data or generator of the Feagi Simple Neuron Model, for a given quantization
#[cfg(feature = "alloc")]
#[derive(Debug, Clone)]
pub enum NMFeagiSimpleNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ: FeagiIndexQuantization> {
    Standard32Bit(ValidatedNeurogenesisStorageOrGenerator<FIQ, NMFeagiSimpleQuantStandard32Bit>),
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> NMFeagiSimpleNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ> {
    /// The quantization of the data held
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

    type NeurogenesisScratchVarianceDefaults<FIQ: FeagiIndexQuantization> =
        super::neurogenesis::NMFeagiSimpleScratchVarianceGenerator<FIQ, Self>;

    type NeurogenesisByGenerator<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorUniformDefault<FIQ, Self>;

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
    type Error = NeuronModelEnumConversionError<NMKind>;

    fn try_from(value: NMKind) -> Result<Self, Self::Error> {
        match value {
            NMKind::FeagiSimple(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleModel)
            }
        }
    }
}

impl From<NMFeagiSimpleQuantedKind> for NMQuantedKind {
    fn from(value: NMFeagiSimpleQuantedKind) -> Self {
        NMQuantedKind::FeagiSimple(value)
    }
}

impl TryFrom<NMQuantedKind> for NMFeagiSimpleQuantedKind {
    type Error = NeuronModelEnumConversionError<NMKind>;

    fn try_from(value: NMQuantedKind) -> Result<Self, Self::Error> {
        match value {
            NMQuantedKind::FeagiSimple(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleModel)
            }
        }
    }
}

impl<FIQ: FeagiIndexQuantization> From<NMFeagiSimpleNeurogenesisGeneratorQuanted<FIQ>> for NMNeurogenesisGeneratorModeled<FIQ> {
    fn from(value: NMFeagiSimpleNeurogenesisGeneratorQuanted<FIQ>) -> Self {
        Self::FeagiSimple(value)
    }
}

impl<FIQ: FeagiIndexQuantization> TryFrom<NMNeurogenesisGeneratorModeled<FIQ>> for NMFeagiSimpleNeurogenesisGeneratorQuanted<FIQ> {
    type Error = NeuronModelEnumConversionError<NMKind>;

    fn try_from(value: NMNeurogenesisGeneratorModeled<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMNeurogenesisGeneratorModeled::FeagiSimple(v) => Ok(v),
            _ => Err(NeuronModelEnumConversionError::IncompatibleModel),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<NMFeagiSimpleNeurogenesisStorageQuanted<FIQ>> for NMNeurogenesisStorageModeled<FIQ> {
    fn from(value: NMFeagiSimpleNeurogenesisStorageQuanted<FIQ>) -> Self {
        Self::FeagiSimple(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMNeurogenesisStorageModeled<FIQ>> for NMFeagiSimpleNeurogenesisStorageQuanted<FIQ> {
    type Error = NeuronModelEnumConversionError<NMKind>;

    fn try_from(value: NMNeurogenesisStorageModeled<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMNeurogenesisStorageModeled::FeagiSimple(v) => Ok(v),
            _ => Err(NeuronModelEnumConversionError::IncompatibleModel),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<NMFeagiSimpleNeurogenesisStorageOrGeneratorQuanted<FIQ>> for NMNeurogenesisStorageOrGeneratorModeled<FIQ> {
    fn from(value: NMFeagiSimpleNeurogenesisStorageOrGeneratorQuanted<FIQ>) -> Self {
        Self::FeagiSimple(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMNeurogenesisStorageOrGeneratorModeled<FIQ>> for NMFeagiSimpleNeurogenesisStorageOrGeneratorQuanted<FIQ> {
    type Error = NeuronModelEnumConversionError<NMKind>;

    fn try_from(value: NMNeurogenesisStorageOrGeneratorModeled<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMNeurogenesisStorageOrGeneratorModeled::FeagiSimple(v) => Ok(v),
            _ => Err(NeuronModelEnumConversionError::IncompatibleModel),
        }
    }
}

impl<FIQ: FeagiIndexQuantization> From<NMFeagiSimpleNeurogenesisValidatedGeneratorQuanted<FIQ>> for NMNeurogenesisValidatedGeneratorModeled<FIQ> {
    fn from(value: NMFeagiSimpleNeurogenesisValidatedGeneratorQuanted<FIQ>) -> Self {
        Self::FeagiSimple(value)
    }
}

impl<FIQ: FeagiIndexQuantization> TryFrom<NMNeurogenesisValidatedGeneratorModeled<FIQ>> for NMFeagiSimpleNeurogenesisValidatedGeneratorQuanted<FIQ> {
    type Error = NeuronModelEnumConversionError<NMKind>;

    fn try_from(value: NMNeurogenesisValidatedGeneratorModeled<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMNeurogenesisValidatedGeneratorModeled::FeagiSimple(v) => Ok(v),
            _ => Err(NeuronModelEnumConversionError::IncompatibleModel),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<NMFeagiSimpleNeurogenesisValidatedStorageQuanted<FIQ>> for NMNeurogenesisValidatedStorageModeled<FIQ> {
    fn from(value: NMFeagiSimpleNeurogenesisValidatedStorageQuanted<FIQ>) -> Self {
        Self::FeagiSimple(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMNeurogenesisValidatedStorageModeled<FIQ>> for NMFeagiSimpleNeurogenesisValidatedStorageQuanted<FIQ> {
    type Error = NeuronModelEnumConversionError<NMKind>;

    fn try_from(value: NMNeurogenesisValidatedStorageModeled<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMNeurogenesisValidatedStorageModeled::FeagiSimple(v) => Ok(v),
            _ => Err(NeuronModelEnumConversionError::IncompatibleModel),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<NMFeagiSimpleNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ>> for NMNeurogenesisValidatedStorageOrGeneratorModeled<FIQ> {
    fn from(value: NMFeagiSimpleNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ>) -> Self {
        Self::FeagiSimple(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMNeurogenesisValidatedStorageOrGeneratorModeled<FIQ>> for NMFeagiSimpleNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ> {
    type Error = NeuronModelEnumConversionError<NMKind>;

    fn try_from(value: NMNeurogenesisValidatedStorageOrGeneratorModeled<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMNeurogenesisValidatedStorageOrGeneratorModeled::FeagiSimple(v) => Ok(v),
            _ => Err(NeuronModelEnumConversionError::IncompatibleModel),
        }
    }
}

//endregion
