use crate::neuron_models::models::feagi_advanced::{NMFeagiAdvancedDataCorticalInternal, NMFeagiAdvancedDataCorticalProperties, NMFeagiAdvancedDataCorticalScratch, NMFeagiAdvancedDataCorticalVariance, NMFeagiAdvancedDataNeuronInternal, NMFeagiAdvancedDataNeuronScratch, NMFeagiAdvancedDataNeuronVariance, NMFeagiAdvancedTrait};
use crate::neuron_models::models::kind::{NMKind, NMQuantedKind};
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::voxel::NeuronLayoutTagVoxel;
use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome_neural::neural_basis::quantization_levels::{FeagiIndexQuantization, MembranePotentialQuantization};
use crate::neuron_models::models::{NMNeurogenesisGeneratorModeled, NMNeurogenesisValidatedGeneratorModeled, NeuronModelEnumConversionError, NeurogenesisByGeneratorTypeOf};
#[cfg(feature = "alloc")]
use crate::neuron_models::models::{NMNeurogenesisStorageModeled, NMNeurogenesisStorageOrGeneratorModeled, NMNeurogenesisValidatedStorageModeled, NMNeurogenesisValidatedStorageOrGeneratorModeled, NeurogenesisFromStorageOf, NeurogenesisFromStorageOrByGeneratorOf};
use crate::neuron_models::neurogenesis_generator_uniform_defaults::NeurogenesisGeneratorUniformDefault;
use feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::validation::ValidatedNeurogenesisGenerator;
#[cfg(feature = "alloc")]
use feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::validation::{ValidatedNeurogenesisStorage, ValidatedNeurogenesisStorageOrGenerator};

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


//region Neurogenesis

/// The neurogenesis generator of the Feagi Advanced Neuron Model, for a given quantization
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(bound = "")]
pub enum NMFeagiAdvancedNeurogenesisGeneratorQuanted<FIQ: FeagiIndexQuantization> {
    Standard32Bit(NeurogenesisByGeneratorTypeOf<FIQ, NMFeagiAdvancedQuantStandard32Bit>),
}

impl<FIQ: FeagiIndexQuantization> NMFeagiAdvancedNeurogenesisGeneratorQuanted<FIQ> {
    /// The quantization of the data held
    pub const fn quant_kind(&self) -> NMFeagiAdvancedQuantKind {
        match self {
            Self::Standard32Bit(_) => NMFeagiAdvancedQuantKind::Standard32Bit,
        }
    }
}

/// The stored neuron data of the Feagi Advanced Neuron Model, for a given quantization
#[cfg(feature = "alloc")]
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(bound = "")]
pub enum NMFeagiAdvancedNeurogenesisStorageQuanted<FIQ: FeagiIndexQuantization> {
    Standard32Bit(NeurogenesisFromStorageOf<FIQ, NMFeagiAdvancedQuantStandard32Bit>),
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> NMFeagiAdvancedNeurogenesisStorageQuanted<FIQ> {
    /// The quantization of the data held
    pub const fn quant_kind(&self) -> NMFeagiAdvancedQuantKind {
        match self {
            Self::Standard32Bit(_) => NMFeagiAdvancedQuantKind::Standard32Bit,
        }
    }
}

/// The stored neuron data or the generator of the Feagi Advanced Neuron Model, for a given quantization
#[cfg(feature = "alloc")]
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(bound = "")]
pub enum NMFeagiAdvancedNeurogenesisStorageOrGeneratorQuanted<FIQ: FeagiIndexQuantization> {
    Standard32Bit(NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMFeagiAdvancedQuantStandard32Bit>),
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> NMFeagiAdvancedNeurogenesisStorageOrGeneratorQuanted<FIQ> {
    /// The quantization of the data held
    pub const fn quant_kind(&self) -> NMFeagiAdvancedQuantKind {
        match self {
            Self::Standard32Bit(_) => NMFeagiAdvancedQuantKind::Standard32Bit,
        }
    }
}

/// The validated neurogenesis generator of the Feagi Advanced Neuron Model, for a given quantization
#[derive(Debug, Clone)]
pub enum NMFeagiAdvancedNeurogenesisValidatedGeneratorQuanted<FIQ: FeagiIndexQuantization> {
    Standard32Bit(ValidatedNeurogenesisGenerator<FIQ, NMFeagiAdvancedQuantStandard32Bit>),
}

impl<FIQ: FeagiIndexQuantization> NMFeagiAdvancedNeurogenesisValidatedGeneratorQuanted<FIQ> {
    /// The quantization of the data held
    pub const fn quant_kind(&self) -> NMFeagiAdvancedQuantKind {
        match self {
            Self::Standard32Bit(_) => NMFeagiAdvancedQuantKind::Standard32Bit,
        }
    }
}

/// The validated stored neuron data of the Feagi Advanced Neuron Model, for a given quantization
#[cfg(feature = "alloc")]
#[derive(Debug, Clone)]
pub enum NMFeagiAdvancedNeurogenesisValidatedStorageQuanted<FIQ: FeagiIndexQuantization> {
    Standard32Bit(ValidatedNeurogenesisStorage<FIQ, NMFeagiAdvancedQuantStandard32Bit>),
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> NMFeagiAdvancedNeurogenesisValidatedStorageQuanted<FIQ> {
    /// The quantization of the data held
    pub const fn quant_kind(&self) -> NMFeagiAdvancedQuantKind {
        match self {
            Self::Standard32Bit(_) => NMFeagiAdvancedQuantKind::Standard32Bit,
        }
    }
}

/// The validated stored neuron data or generator of the Feagi Advanced Neuron Model, for a given quantization
#[cfg(feature = "alloc")]
#[derive(Debug, Clone)]
pub enum NMFeagiAdvancedNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ: FeagiIndexQuantization> {
    Standard32Bit(ValidatedNeurogenesisStorageOrGenerator<FIQ, NMFeagiAdvancedQuantStandard32Bit>),
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> NMFeagiAdvancedNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ> {
    /// The quantization of the data held
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

    type NeurogenesisByGenerator<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorUniformDefault<FIQ, Self>;

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

impl<FIQ: FeagiIndexQuantization> From<NMFeagiAdvancedNeurogenesisGeneratorQuanted<FIQ>> for NMNeurogenesisGeneratorModeled<FIQ> {
    fn from(value: NMFeagiAdvancedNeurogenesisGeneratorQuanted<FIQ>) -> Self {
        Self::FeagiAdvanced(value)
    }
}

impl<FIQ: FeagiIndexQuantization> TryFrom<NMNeurogenesisGeneratorModeled<FIQ>> for NMFeagiAdvancedNeurogenesisGeneratorQuanted<FIQ> {
    type Error = NeuronModelEnumConversionError<NMKind>;

    fn try_from(value: NMNeurogenesisGeneratorModeled<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMNeurogenesisGeneratorModeled::FeagiAdvanced(v) => Ok(v),
            _ => Err(NeuronModelEnumConversionError::IncompatibleModel),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<NMFeagiAdvancedNeurogenesisStorageQuanted<FIQ>> for NMNeurogenesisStorageModeled<FIQ> {
    fn from(value: NMFeagiAdvancedNeurogenesisStorageQuanted<FIQ>) -> Self {
        Self::FeagiAdvanced(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMNeurogenesisStorageModeled<FIQ>> for NMFeagiAdvancedNeurogenesisStorageQuanted<FIQ> {
    type Error = NeuronModelEnumConversionError<NMKind>;

    fn try_from(value: NMNeurogenesisStorageModeled<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMNeurogenesisStorageModeled::FeagiAdvanced(v) => Ok(v),
            _ => Err(NeuronModelEnumConversionError::IncompatibleModel),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<NMFeagiAdvancedNeurogenesisStorageOrGeneratorQuanted<FIQ>> for NMNeurogenesisStorageOrGeneratorModeled<FIQ> {
    fn from(value: NMFeagiAdvancedNeurogenesisStorageOrGeneratorQuanted<FIQ>) -> Self {
        Self::FeagiAdvanced(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMNeurogenesisStorageOrGeneratorModeled<FIQ>> for NMFeagiAdvancedNeurogenesisStorageOrGeneratorQuanted<FIQ> {
    type Error = NeuronModelEnumConversionError<NMKind>;

    fn try_from(value: NMNeurogenesisStorageOrGeneratorModeled<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMNeurogenesisStorageOrGeneratorModeled::FeagiAdvanced(v) => Ok(v),
            _ => Err(NeuronModelEnumConversionError::IncompatibleModel),
        }
    }
}

impl<FIQ: FeagiIndexQuantization> From<NMFeagiAdvancedNeurogenesisValidatedGeneratorQuanted<FIQ>> for NMNeurogenesisValidatedGeneratorModeled<FIQ> {
    fn from(value: NMFeagiAdvancedNeurogenesisValidatedGeneratorQuanted<FIQ>) -> Self {
        Self::FeagiAdvanced(value)
    }
}

impl<FIQ: FeagiIndexQuantization> TryFrom<NMNeurogenesisValidatedGeneratorModeled<FIQ>> for NMFeagiAdvancedNeurogenesisValidatedGeneratorQuanted<FIQ> {
    type Error = NeuronModelEnumConversionError<NMKind>;

    fn try_from(value: NMNeurogenesisValidatedGeneratorModeled<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMNeurogenesisValidatedGeneratorModeled::FeagiAdvanced(v) => Ok(v),
            _ => Err(NeuronModelEnumConversionError::IncompatibleModel),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<NMFeagiAdvancedNeurogenesisValidatedStorageQuanted<FIQ>> for NMNeurogenesisValidatedStorageModeled<FIQ> {
    fn from(value: NMFeagiAdvancedNeurogenesisValidatedStorageQuanted<FIQ>) -> Self {
        Self::FeagiAdvanced(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMNeurogenesisValidatedStorageModeled<FIQ>> for NMFeagiAdvancedNeurogenesisValidatedStorageQuanted<FIQ> {
    type Error = NeuronModelEnumConversionError<NMKind>;

    fn try_from(value: NMNeurogenesisValidatedStorageModeled<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMNeurogenesisValidatedStorageModeled::FeagiAdvanced(v) => Ok(v),
            _ => Err(NeuronModelEnumConversionError::IncompatibleModel),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<NMFeagiAdvancedNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ>> for NMNeurogenesisValidatedStorageOrGeneratorModeled<FIQ> {
    fn from(value: NMFeagiAdvancedNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ>) -> Self {
        Self::FeagiAdvanced(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMNeurogenesisValidatedStorageOrGeneratorModeled<FIQ>> for NMFeagiAdvancedNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ> {
    type Error = NeuronModelEnumConversionError<NMKind>;

    fn try_from(value: NMNeurogenesisValidatedStorageOrGeneratorModeled<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMNeurogenesisValidatedStorageOrGeneratorModeled::FeagiAdvanced(v) => Ok(v),
            _ => Err(NeuronModelEnumConversionError::IncompatibleModel),
        }
    }
}

//endregion
