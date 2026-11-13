use crate::neuron_models::models::memory_fire_hashing::{NMMemoryFireHashingDataCorticalInternal, NMMemoryFireHashingDataCorticalProperties, NMMemoryFireHashingDataCorticalScratch, NMMemoryFireHashingDataCorticalVariance, NMMemoryFireHashingDataNeuronInternal, NMMemoryFireHashingDataNeuronScratch, NMMemoryFireHashingDataNeuronVariance, NMMemoryFireHashingTrait};
use crate::neuron_models::models::kind::{NMKind, NMQuantedKind};
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::single_stepped_dynamic::NeuronLayoutTagSingleSteppedDynamic;
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


//region Neurogenesis

/// The neurogenesis generator of the Memory Fire Hashing Neuron Model, for a given quantization
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(bound = "")]
pub enum NMMemoryFireHashingNeurogenesisGeneratorQuanted<FIQ: FeagiIndexQuantization> {
    Standard32Bit(NeurogenesisByGeneratorTypeOf<FIQ, NMMemoryFireHashingQuantStandard32Bit>),
}

impl<FIQ: FeagiIndexQuantization> NMMemoryFireHashingNeurogenesisGeneratorQuanted<FIQ> {
    /// The quantization of the data held
    pub const fn quant_kind(&self) -> NMMemoryFireHashingQuantKind {
        match self {
            Self::Standard32Bit(_) => NMMemoryFireHashingQuantKind::Standard32Bit,
        }
    }
}

/// The stored neuron data of the Memory Fire Hashing Neuron Model, for a given quantization
#[cfg(feature = "alloc")]
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(bound = "")]
pub enum NMMemoryFireHashingNeurogenesisStorageQuanted<FIQ: FeagiIndexQuantization> {
    Standard32Bit(NeurogenesisFromStorageOf<FIQ, NMMemoryFireHashingQuantStandard32Bit>),
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> NMMemoryFireHashingNeurogenesisStorageQuanted<FIQ> {
    /// The quantization of the data held
    pub const fn quant_kind(&self) -> NMMemoryFireHashingQuantKind {
        match self {
            Self::Standard32Bit(_) => NMMemoryFireHashingQuantKind::Standard32Bit,
        }
    }
}

/// The stored neuron data or the generator of the Memory Fire Hashing Neuron Model, for a given quantization
#[cfg(feature = "alloc")]
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(bound = "")]
pub enum NMMemoryFireHashingNeurogenesisStorageOrGeneratorQuanted<FIQ: FeagiIndexQuantization> {
    Standard32Bit(NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMMemoryFireHashingQuantStandard32Bit>),
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> NMMemoryFireHashingNeurogenesisStorageOrGeneratorQuanted<FIQ> {
    /// The quantization of the data held
    pub const fn quant_kind(&self) -> NMMemoryFireHashingQuantKind {
        match self {
            Self::Standard32Bit(_) => NMMemoryFireHashingQuantKind::Standard32Bit,
        }
    }
}

/// The validated neurogenesis generator of the Memory Fire Hashing Neuron Model, for a given quantization
#[derive(Debug, Clone)]
pub enum NMMemoryFireHashingNeurogenesisValidatedGeneratorQuanted<FIQ: FeagiIndexQuantization> {
    Standard32Bit(ValidatedNeurogenesisGenerator<FIQ, NMMemoryFireHashingQuantStandard32Bit>),
}

impl<FIQ: FeagiIndexQuantization> NMMemoryFireHashingNeurogenesisValidatedGeneratorQuanted<FIQ> {
    /// The quantization of the data held
    pub const fn quant_kind(&self) -> NMMemoryFireHashingQuantKind {
        match self {
            Self::Standard32Bit(_) => NMMemoryFireHashingQuantKind::Standard32Bit,
        }
    }
}

/// The validated stored neuron data of the Memory Fire Hashing Neuron Model, for a given quantization
#[cfg(feature = "alloc")]
#[derive(Debug, Clone)]
pub enum NMMemoryFireHashingNeurogenesisValidatedStorageQuanted<FIQ: FeagiIndexQuantization> {
    Standard32Bit(ValidatedNeurogenesisStorage<FIQ, NMMemoryFireHashingQuantStandard32Bit>),
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> NMMemoryFireHashingNeurogenesisValidatedStorageQuanted<FIQ> {
    /// The quantization of the data held
    pub const fn quant_kind(&self) -> NMMemoryFireHashingQuantKind {
        match self {
            Self::Standard32Bit(_) => NMMemoryFireHashingQuantKind::Standard32Bit,
        }
    }
}

/// The validated stored neuron data or generator of the Memory Fire Hashing Neuron Model, for a given quantization
#[cfg(feature = "alloc")]
#[derive(Debug, Clone)]
pub enum NMMemoryFireHashingNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ: FeagiIndexQuantization> {
    Standard32Bit(ValidatedNeurogenesisStorageOrGenerator<FIQ, NMMemoryFireHashingQuantStandard32Bit>),
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> NMMemoryFireHashingNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ> {
    /// The quantization of the data held
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

    type NeurogenesisScratchVarianceDefaults<FIQ: FeagiIndexQuantization> =
        super::neurogenesis::MemoryFireHashingNeurogenesisDefaults<FIQ, Self>;

    type NeurogenesisByGenerator<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorUniformDefault<FIQ, Self>;

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

impl<FIQ: FeagiIndexQuantization> From<NMMemoryFireHashingNeurogenesisGeneratorQuanted<FIQ>> for NMNeurogenesisGeneratorModeled<FIQ> {
    fn from(value: NMMemoryFireHashingNeurogenesisGeneratorQuanted<FIQ>) -> Self {
        Self::MemoryFireHashing(value)
    }
}

impl<FIQ: FeagiIndexQuantization> TryFrom<NMNeurogenesisGeneratorModeled<FIQ>> for NMMemoryFireHashingNeurogenesisGeneratorQuanted<FIQ> {
    type Error = NeuronModelEnumConversionError<NMKind>;

    fn try_from(value: NMNeurogenesisGeneratorModeled<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMNeurogenesisGeneratorModeled::MemoryFireHashing(v) => Ok(v),
            _ => Err(NeuronModelEnumConversionError::IncompatibleModel),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<NMMemoryFireHashingNeurogenesisStorageQuanted<FIQ>> for NMNeurogenesisStorageModeled<FIQ> {
    fn from(value: NMMemoryFireHashingNeurogenesisStorageQuanted<FIQ>) -> Self {
        Self::MemoryFireHashing(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMNeurogenesisStorageModeled<FIQ>> for NMMemoryFireHashingNeurogenesisStorageQuanted<FIQ> {
    type Error = NeuronModelEnumConversionError<NMKind>;

    fn try_from(value: NMNeurogenesisStorageModeled<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMNeurogenesisStorageModeled::MemoryFireHashing(v) => Ok(v),
            _ => Err(NeuronModelEnumConversionError::IncompatibleModel),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<NMMemoryFireHashingNeurogenesisStorageOrGeneratorQuanted<FIQ>> for NMNeurogenesisStorageOrGeneratorModeled<FIQ> {
    fn from(value: NMMemoryFireHashingNeurogenesisStorageOrGeneratorQuanted<FIQ>) -> Self {
        Self::MemoryFireHashing(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMNeurogenesisStorageOrGeneratorModeled<FIQ>> for NMMemoryFireHashingNeurogenesisStorageOrGeneratorQuanted<FIQ> {
    type Error = NeuronModelEnumConversionError<NMKind>;

    fn try_from(value: NMNeurogenesisStorageOrGeneratorModeled<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMNeurogenesisStorageOrGeneratorModeled::MemoryFireHashing(v) => Ok(v),
            _ => Err(NeuronModelEnumConversionError::IncompatibleModel),
        }
    }
}

impl<FIQ: FeagiIndexQuantization> From<NMMemoryFireHashingNeurogenesisValidatedGeneratorQuanted<FIQ>> for NMNeurogenesisValidatedGeneratorModeled<FIQ> {
    fn from(value: NMMemoryFireHashingNeurogenesisValidatedGeneratorQuanted<FIQ>) -> Self {
        Self::MemoryFireHashing(value)
    }
}

impl<FIQ: FeagiIndexQuantization> TryFrom<NMNeurogenesisValidatedGeneratorModeled<FIQ>> for NMMemoryFireHashingNeurogenesisValidatedGeneratorQuanted<FIQ> {
    type Error = NeuronModelEnumConversionError<NMKind>;

    fn try_from(value: NMNeurogenesisValidatedGeneratorModeled<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMNeurogenesisValidatedGeneratorModeled::MemoryFireHashing(v) => Ok(v),
            _ => Err(NeuronModelEnumConversionError::IncompatibleModel),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<NMMemoryFireHashingNeurogenesisValidatedStorageQuanted<FIQ>> for NMNeurogenesisValidatedStorageModeled<FIQ> {
    fn from(value: NMMemoryFireHashingNeurogenesisValidatedStorageQuanted<FIQ>) -> Self {
        Self::MemoryFireHashing(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMNeurogenesisValidatedStorageModeled<FIQ>> for NMMemoryFireHashingNeurogenesisValidatedStorageQuanted<FIQ> {
    type Error = NeuronModelEnumConversionError<NMKind>;

    fn try_from(value: NMNeurogenesisValidatedStorageModeled<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMNeurogenesisValidatedStorageModeled::MemoryFireHashing(v) => Ok(v),
            _ => Err(NeuronModelEnumConversionError::IncompatibleModel),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<NMMemoryFireHashingNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ>> for NMNeurogenesisValidatedStorageOrGeneratorModeled<FIQ> {
    fn from(value: NMMemoryFireHashingNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ>) -> Self {
        Self::MemoryFireHashing(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMNeurogenesisValidatedStorageOrGeneratorModeled<FIQ>> for NMMemoryFireHashingNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ> {
    type Error = NeuronModelEnumConversionError<NMKind>;

    fn try_from(value: NMNeurogenesisValidatedStorageOrGeneratorModeled<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMNeurogenesisValidatedStorageOrGeneratorModeled::MemoryFireHashing(v) => Ok(v),
            _ => Err(NeuronModelEnumConversionError::IncompatibleModel),
        }
    }
}

//endregion
