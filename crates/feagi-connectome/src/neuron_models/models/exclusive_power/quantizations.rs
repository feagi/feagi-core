use crate::neuron_models::models::exclusive_power::{NMExclusivePowerDataCorticalInternal, NMExclusivePowerDataCorticalProperties, NMExclusivePowerDataCorticalScratch, NMExclusivePowerDataCorticalVariance, NMExclusivePowerDataNeuronInternal, NMExclusivePowerDataNeuronScratch, NMExclusivePowerDataNeuronVariance, NMExclusivePowerTrait};
use crate::neuron_models::models::kind::{NMKind, NMQuantedKind};
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::voxel::NeuronLayoutTagVoxel;
use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome_neural::neural_basis::quantization_levels::{FeagiIndexQuantization, MembranePotentialQuantization};
use half::{bf16, f16};
use crate::neuron_models::models::{NMNeurogenesisGeneratorModeled, NMNeurogenesisValidatedGeneratorModeled, NeuronModelEnumConversionError, NeurogenesisByGeneratorTypeOf};
#[cfg(feature = "alloc")]
use crate::neuron_models::models::{NMNeurogenesisStorageModeled, NMNeurogenesisStorageOrGeneratorModeled, NMNeurogenesisValidatedStorageModeled, NMNeurogenesisValidatedStorageOrGeneratorModeled, NeurogenesisFromStorageOf, NeurogenesisFromStorageOrByGeneratorOf};
use crate::neuron_models::neurogenesis_generator_uniform_defaults::NeurogenesisGeneratorUniformDefault;
use feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::validation::ValidatedNeurogenesisGenerator;
#[cfg(feature = "alloc")]
use feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::validation::{ValidatedNeurogenesisStorage, ValidatedNeurogenesisStorageOrGenerator};

//region enums

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
/// Describes the quantization of the Exclusive Power Neuron Model
pub enum NMExclusivePowerQuantKind {
    Float16,
    BFloat16,
    Float32,
    Float64,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum NMExclusivePowerQuantedKind {
    Float16(NMExclusivePowerQuantFloat16),
    BFloat16(NMExclusivePowerQuantBFloat16),
    Float32(NMExclusivePowerQuantFloat32),
    Float64(NMExclusivePowerQuantFloat64)
}

impl NMExclusivePowerQuantedKind {
    pub fn quant_kind(&self) -> NMExclusivePowerQuantKind {
        match &self {
            NMExclusivePowerQuantedKind::Float16(_) => NMExclusivePowerQuantKind::Float16,
            NMExclusivePowerQuantedKind::BFloat16(_) => NMExclusivePowerQuantKind::BFloat16,
            NMExclusivePowerQuantedKind::Float32(_) => NMExclusivePowerQuantKind::Float32,
            NMExclusivePowerQuantedKind::Float64(_) => NMExclusivePowerQuantKind::Float64,
        }
    }

    pub fn from_nm_quant<NM: NMExclusivePowerTrait>() -> NMExclusivePowerQuantedKind {
        match NM::QUANT_KIND {
            NMExclusivePowerQuantKind::Float16 => NMExclusivePowerQuantedKind::Float16(NMExclusivePowerQuantFloat16),
            NMExclusivePowerQuantKind::BFloat16 => NMExclusivePowerQuantedKind::BFloat16(NMExclusivePowerQuantBFloat16),
            NMExclusivePowerQuantKind::Float32 => NMExclusivePowerQuantedKind::Float32(NMExclusivePowerQuantFloat32),
            NMExclusivePowerQuantKind::Float64 => NMExclusivePowerQuantedKind::Float64(NMExclusivePowerQuantFloat64),
        }
    }
}


//region Neurogenesis

/// The neurogenesis generator of the Exclusive Power Neuron Model, for a given quantization
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(bound = "")]
pub enum NMExclusivePowerNeurogenesisGeneratorQuanted<FIQ: FeagiIndexQuantization> {
    Float16(NeurogenesisByGeneratorTypeOf<FIQ, NMExclusivePowerQuantFloat16>),
    BFloat16(NeurogenesisByGeneratorTypeOf<FIQ, NMExclusivePowerQuantBFloat16>),
    Float32(NeurogenesisByGeneratorTypeOf<FIQ, NMExclusivePowerQuantFloat32>),
    Float64(NeurogenesisByGeneratorTypeOf<FIQ, NMExclusivePowerQuantFloat64>),
}

impl<FIQ: FeagiIndexQuantization> NMExclusivePowerNeurogenesisGeneratorQuanted<FIQ> {
    /// The quantization of the data held
    pub const fn quant_kind(&self) -> NMExclusivePowerQuantKind {
        match self {
            Self::Float16(_) => NMExclusivePowerQuantKind::Float16,
            Self::BFloat16(_) => NMExclusivePowerQuantKind::BFloat16,
            Self::Float32(_) => NMExclusivePowerQuantKind::Float32,
            Self::Float64(_) => NMExclusivePowerQuantKind::Float64,
        }
    }
}

/// The stored neuron data of the Exclusive Power Neuron Model, for a given quantization
#[cfg(feature = "alloc")]
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(bound = "")]
pub enum NMExclusivePowerNeurogenesisStorageQuanted<FIQ: FeagiIndexQuantization> {
    Float16(NeurogenesisFromStorageOf<FIQ, NMExclusivePowerQuantFloat16>),
    BFloat16(NeurogenesisFromStorageOf<FIQ, NMExclusivePowerQuantBFloat16>),
    Float32(NeurogenesisFromStorageOf<FIQ, NMExclusivePowerQuantFloat32>),
    Float64(NeurogenesisFromStorageOf<FIQ, NMExclusivePowerQuantFloat64>),
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> NMExclusivePowerNeurogenesisStorageQuanted<FIQ> {
    /// The quantization of the data held
    pub const fn quant_kind(&self) -> NMExclusivePowerQuantKind {
        match self {
            Self::Float16(_) => NMExclusivePowerQuantKind::Float16,
            Self::BFloat16(_) => NMExclusivePowerQuantKind::BFloat16,
            Self::Float32(_) => NMExclusivePowerQuantKind::Float32,
            Self::Float64(_) => NMExclusivePowerQuantKind::Float64,
        }
    }
}

/// The stored neuron data or the generator of the Exclusive Power Neuron Model, for a given quantization
#[cfg(feature = "alloc")]
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(bound = "")]
pub enum NMExclusivePowerNeurogenesisStorageOrGeneratorQuanted<FIQ: FeagiIndexQuantization> {
    Float16(NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMExclusivePowerQuantFloat16>),
    BFloat16(NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMExclusivePowerQuantBFloat16>),
    Float32(NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMExclusivePowerQuantFloat32>),
    Float64(NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMExclusivePowerQuantFloat64>),
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> NMExclusivePowerNeurogenesisStorageOrGeneratorQuanted<FIQ> {
    /// The quantization of the data held
    pub const fn quant_kind(&self) -> NMExclusivePowerQuantKind {
        match self {
            Self::Float16(_) => NMExclusivePowerQuantKind::Float16,
            Self::BFloat16(_) => NMExclusivePowerQuantKind::BFloat16,
            Self::Float32(_) => NMExclusivePowerQuantKind::Float32,
            Self::Float64(_) => NMExclusivePowerQuantKind::Float64,
        }
    }
}

/// The validated neurogenesis generator of the Exclusive Power Neuron Model, for a given quantization
#[derive(Debug, Clone)]
pub enum NMExclusivePowerNeurogenesisValidatedGeneratorQuanted<FIQ: FeagiIndexQuantization> {
    Float16(ValidatedNeurogenesisGenerator<FIQ, NMExclusivePowerQuantFloat16>),
    BFloat16(ValidatedNeurogenesisGenerator<FIQ, NMExclusivePowerQuantBFloat16>),
    Float32(ValidatedNeurogenesisGenerator<FIQ, NMExclusivePowerQuantFloat32>),
    Float64(ValidatedNeurogenesisGenerator<FIQ, NMExclusivePowerQuantFloat64>),
}

impl<FIQ: FeagiIndexQuantization> NMExclusivePowerNeurogenesisValidatedGeneratorQuanted<FIQ> {
    /// The quantization of the data held
    pub const fn quant_kind(&self) -> NMExclusivePowerQuantKind {
        match self {
            Self::Float16(_) => NMExclusivePowerQuantKind::Float16,
            Self::BFloat16(_) => NMExclusivePowerQuantKind::BFloat16,
            Self::Float32(_) => NMExclusivePowerQuantKind::Float32,
            Self::Float64(_) => NMExclusivePowerQuantKind::Float64,
        }
    }
}

/// The validated stored neuron data of the Exclusive Power Neuron Model, for a given quantization
#[cfg(feature = "alloc")]
#[derive(Debug, Clone)]
pub enum NMExclusivePowerNeurogenesisValidatedStorageQuanted<FIQ: FeagiIndexQuantization> {
    Float16(ValidatedNeurogenesisStorage<FIQ, NMExclusivePowerQuantFloat16>),
    BFloat16(ValidatedNeurogenesisStorage<FIQ, NMExclusivePowerQuantBFloat16>),
    Float32(ValidatedNeurogenesisStorage<FIQ, NMExclusivePowerQuantFloat32>),
    Float64(ValidatedNeurogenesisStorage<FIQ, NMExclusivePowerQuantFloat64>),
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> NMExclusivePowerNeurogenesisValidatedStorageQuanted<FIQ> {
    /// The quantization of the data held
    pub const fn quant_kind(&self) -> NMExclusivePowerQuantKind {
        match self {
            Self::Float16(_) => NMExclusivePowerQuantKind::Float16,
            Self::BFloat16(_) => NMExclusivePowerQuantKind::BFloat16,
            Self::Float32(_) => NMExclusivePowerQuantKind::Float32,
            Self::Float64(_) => NMExclusivePowerQuantKind::Float64,
        }
    }
}

/// The validated stored neuron data or generator of the Exclusive Power Neuron Model, for a given quantization
#[cfg(feature = "alloc")]
#[derive(Debug, Clone)]
pub enum NMExclusivePowerNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ: FeagiIndexQuantization> {
    Float16(ValidatedNeurogenesisStorageOrGenerator<FIQ, NMExclusivePowerQuantFloat16>),
    BFloat16(ValidatedNeurogenesisStorageOrGenerator<FIQ, NMExclusivePowerQuantBFloat16>),
    Float32(ValidatedNeurogenesisStorageOrGenerator<FIQ, NMExclusivePowerQuantFloat32>),
    Float64(ValidatedNeurogenesisStorageOrGenerator<FIQ, NMExclusivePowerQuantFloat64>),
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> NMExclusivePowerNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ> {
    /// The quantization of the data held
    pub const fn quant_kind(&self) -> NMExclusivePowerQuantKind {
        match self {
            Self::Float16(_) => NMExclusivePowerQuantKind::Float16,
            Self::BFloat16(_) => NMExclusivePowerQuantKind::BFloat16,
            Self::Float32(_) => NMExclusivePowerQuantKind::Float32,
            Self::Float64(_) => NMExclusivePowerQuantKind::Float64,
        }
    }
}

//endregion

//endregion

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMExclusivePowerQuantFloat16;

impl NMExclusivePowerTrait for NMExclusivePowerQuantFloat16 {
}

impl NeuronModel for NMExclusivePowerQuantFloat16 {
    type NeuronLayoutTag = NeuronLayoutTagVoxel;

    type NeurogenesisScratchVarianceDefaults<FIQ: FeagiIndexQuantization> =
        super::neurogenesis::ExclusivePowerNeurogenesisDefaults<FIQ, Self>;

    type NeurogenesisByGenerator<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorUniformDefault<FIQ, Self>;

    type CorticalDataProperties = NMExclusivePowerDataCorticalProperties<Self>;
    type CorticalDataInternal = NMExclusivePowerDataCorticalInternal<Self>;
    type CorticalDataScratch = NMExclusivePowerDataCorticalScratch<Self>;
    type CorticalDataVariance = NMExclusivePowerDataCorticalVariance<Self>;
    type NeuronDataInternal = NMExclusivePowerDataNeuronInternal<Self>;
    type NeuronDataScratch = NMExclusivePowerDataNeuronScratch<Self>;
    type NeuronDataVariance = NMExclusivePowerDataNeuronVariance<Self>;

    type QuantKind = NMExclusivePowerQuantKind;
    type QuantedKind = NMExclusivePowerQuantedKind;
    const QUANT_KIND: Self::QuantKind = NMExclusivePowerQuantKind::Float16;
    const QUANTED_KIND: Self::QuantedKind = NMExclusivePowerQuantedKind::Float16(Self);
}

impl MembranePotentialQuantization for NMExclusivePowerQuantFloat16
{
    type MembranePotentialQuant = f16;
}

impl From<NMExclusivePowerQuantFloat16> for NMExclusivePowerQuantedKind {
    fn from(value: NMExclusivePowerQuantFloat16) -> Self {
        NMExclusivePowerQuantedKind::Float16(value)
    }
}

impl TryFrom<NMExclusivePowerQuantedKind> for NMExclusivePowerQuantFloat16 {
    type Error = NeuronModelEnumConversionError<NMExclusivePowerQuantKind>;

    fn try_from(value: NMExclusivePowerQuantedKind) -> Result<Self, Self::Error> {
        let actual = value.quant_kind();
        match value {
            NMExclusivePowerQuantedKind::Float16(s) => {Ok(s)},
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMExclusivePowerQuantKind::Float16,
                    actual,
                })
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMExclusivePowerQuantBFloat16;

impl NMExclusivePowerTrait for NMExclusivePowerQuantBFloat16 {
}

impl NeuronModel for NMExclusivePowerQuantBFloat16 {
    type NeuronLayoutTag = NeuronLayoutTagVoxel;

    type NeurogenesisScratchVarianceDefaults<FIQ: FeagiIndexQuantization> =
        super::neurogenesis::ExclusivePowerNeurogenesisDefaults<FIQ, Self>;

    type NeurogenesisByGenerator<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorUniformDefault<FIQ, Self>;

    type CorticalDataProperties = NMExclusivePowerDataCorticalProperties<Self>;
    type CorticalDataInternal = NMExclusivePowerDataCorticalInternal<Self>;
    type CorticalDataScratch = NMExclusivePowerDataCorticalScratch<Self>;
    type CorticalDataVariance = NMExclusivePowerDataCorticalVariance<Self>;
    type NeuronDataInternal = NMExclusivePowerDataNeuronInternal<Self>;
    type NeuronDataScratch = NMExclusivePowerDataNeuronScratch<Self>;
    type NeuronDataVariance = NMExclusivePowerDataNeuronVariance<Self>;

    type QuantKind = NMExclusivePowerQuantKind;
    type QuantedKind = NMExclusivePowerQuantedKind;
    const QUANT_KIND: Self::QuantKind = NMExclusivePowerQuantKind::BFloat16;
    const QUANTED_KIND: Self::QuantedKind = NMExclusivePowerQuantedKind::BFloat16(Self);
}

impl MembranePotentialQuantization for NMExclusivePowerQuantBFloat16
{
    type MembranePotentialQuant = bf16;
}

impl From<NMExclusivePowerQuantBFloat16> for NMExclusivePowerQuantedKind {
    fn from(value: NMExclusivePowerQuantBFloat16) -> Self {
        NMExclusivePowerQuantedKind::BFloat16(value)
    }
}

impl TryFrom<NMExclusivePowerQuantedKind> for NMExclusivePowerQuantBFloat16 {
    type Error = NeuronModelEnumConversionError<NMExclusivePowerQuantKind>;

    fn try_from(value: NMExclusivePowerQuantedKind) -> Result<Self, Self::Error> {
        let actual = value.quant_kind();
        match value {
            NMExclusivePowerQuantedKind::BFloat16(s) => {Ok(s)},
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMExclusivePowerQuantKind::BFloat16,
                    actual,
                })
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMExclusivePowerQuantFloat32;

impl NMExclusivePowerTrait for NMExclusivePowerQuantFloat32 {
}

impl NeuronModel for NMExclusivePowerQuantFloat32 {
    type NeuronLayoutTag = NeuronLayoutTagVoxel;

    type NeurogenesisScratchVarianceDefaults<FIQ: FeagiIndexQuantization> =
        super::neurogenesis::ExclusivePowerNeurogenesisDefaults<FIQ, Self>;

    type NeurogenesisByGenerator<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorUniformDefault<FIQ, Self>;

    type CorticalDataProperties = NMExclusivePowerDataCorticalProperties<Self>;
    type CorticalDataInternal = NMExclusivePowerDataCorticalInternal<Self>;
    type CorticalDataScratch = NMExclusivePowerDataCorticalScratch<Self>;
    type CorticalDataVariance = NMExclusivePowerDataCorticalVariance<Self>;
    type NeuronDataInternal = NMExclusivePowerDataNeuronInternal<Self>;
    type NeuronDataScratch = NMExclusivePowerDataNeuronScratch<Self>;
    type NeuronDataVariance = NMExclusivePowerDataNeuronVariance<Self>;

    type QuantKind = NMExclusivePowerQuantKind;
    type QuantedKind = NMExclusivePowerQuantedKind;
    const QUANT_KIND: Self::QuantKind = NMExclusivePowerQuantKind::Float32;
    const QUANTED_KIND: Self::QuantedKind = NMExclusivePowerQuantedKind::Float32(Self);
}

impl MembranePotentialQuantization for NMExclusivePowerQuantFloat32
{
    type MembranePotentialQuant = f32;
}

impl From<NMExclusivePowerQuantFloat32> for NMExclusivePowerQuantedKind {
    fn from(value: NMExclusivePowerQuantFloat32) -> Self {
        NMExclusivePowerQuantedKind::Float32(value)
    }
}

impl TryFrom<NMExclusivePowerQuantedKind> for NMExclusivePowerQuantFloat32 {
    type Error = NeuronModelEnumConversionError<NMExclusivePowerQuantKind>;

    fn try_from(value: NMExclusivePowerQuantedKind) -> Result<Self, Self::Error> {
        let actual = value.quant_kind();
        match value {
            NMExclusivePowerQuantedKind::Float32(s) => {Ok(s)},
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMExclusivePowerQuantKind::Float32,
                    actual,
                })
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMExclusivePowerQuantFloat64;

impl NMExclusivePowerTrait for NMExclusivePowerQuantFloat64 {
}

impl NeuronModel for NMExclusivePowerQuantFloat64 {
    type NeuronLayoutTag = NeuronLayoutTagVoxel;

    type NeurogenesisScratchVarianceDefaults<FIQ: FeagiIndexQuantization> =
        super::neurogenesis::ExclusivePowerNeurogenesisDefaults<FIQ, Self>;

    type NeurogenesisByGenerator<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorUniformDefault<FIQ, Self>;

    type CorticalDataProperties = NMExclusivePowerDataCorticalProperties<Self>;
    type CorticalDataInternal = NMExclusivePowerDataCorticalInternal<Self>;
    type CorticalDataScratch = NMExclusivePowerDataCorticalScratch<Self>;
    type CorticalDataVariance = NMExclusivePowerDataCorticalVariance<Self>;
    type NeuronDataInternal = NMExclusivePowerDataNeuronInternal<Self>;
    type NeuronDataScratch = NMExclusivePowerDataNeuronScratch<Self>;
    type NeuronDataVariance = NMExclusivePowerDataNeuronVariance<Self>;

    type QuantKind = NMExclusivePowerQuantKind;
    type QuantedKind = NMExclusivePowerQuantedKind;
    const QUANT_KIND: Self::QuantKind = NMExclusivePowerQuantKind::Float64;
    const QUANTED_KIND: Self::QuantedKind = NMExclusivePowerQuantedKind::Float64(Self);
}

impl MembranePotentialQuantization for NMExclusivePowerQuantFloat64
{
    type MembranePotentialQuant = f64;
}

impl From<NMExclusivePowerQuantFloat64> for NMExclusivePowerQuantedKind {
    fn from(value: NMExclusivePowerQuantFloat64) -> Self {
        NMExclusivePowerQuantedKind::Float64(value)
    }
}

impl TryFrom<NMExclusivePowerQuantedKind> for NMExclusivePowerQuantFloat64 {
    type Error = NeuronModelEnumConversionError<NMExclusivePowerQuantKind>;

    fn try_from(value: NMExclusivePowerQuantedKind) -> Result<Self, Self::Error> {
        let actual = value.quant_kind();
        match value {
            NMExclusivePowerQuantedKind::Float64(s) => {Ok(s)},
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMExclusivePowerQuantKind::Float64,
                    actual,
                })
            }
        }
    }
}

//region Model Level Conversions

impl From<NMExclusivePowerQuantKind> for NMKind {
    fn from(value: NMExclusivePowerQuantKind) -> Self {
        NMKind::ExclusivePower(value)
    }
}

impl TryFrom<NMKind> for NMExclusivePowerQuantKind {
    type Error = NeuronModelEnumConversionError<NMKind>;

    fn try_from(value: NMKind) -> Result<Self, Self::Error> {
        match value {
            NMKind::ExclusivePower(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleModel)
            }
        }
    }
}

impl From<NMExclusivePowerQuantedKind> for NMQuantedKind {
    fn from(value: NMExclusivePowerQuantedKind) -> Self {
        NMQuantedKind::ExclusivePower(value)
    }
}

impl TryFrom<NMQuantedKind> for NMExclusivePowerQuantedKind {
    type Error = NeuronModelEnumConversionError<NMKind>;

    fn try_from(value: NMQuantedKind) -> Result<Self, Self::Error> {
        match value {
            NMQuantedKind::ExclusivePower(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleModel)
            }
        }
    }
}

impl<FIQ: FeagiIndexQuantization> From<NMExclusivePowerNeurogenesisGeneratorQuanted<FIQ>> for NMNeurogenesisGeneratorModeled<FIQ> {
    fn from(value: NMExclusivePowerNeurogenesisGeneratorQuanted<FIQ>) -> Self {
        Self::ExclusivePower(value)
    }
}

impl<FIQ: FeagiIndexQuantization> TryFrom<NMNeurogenesisGeneratorModeled<FIQ>> for NMExclusivePowerNeurogenesisGeneratorQuanted<FIQ> {
    type Error = NeuronModelEnumConversionError<NMKind>;

    fn try_from(value: NMNeurogenesisGeneratorModeled<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMNeurogenesisGeneratorModeled::ExclusivePower(v) => Ok(v),
            _ => Err(NeuronModelEnumConversionError::IncompatibleModel),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<NMExclusivePowerNeurogenesisStorageQuanted<FIQ>> for NMNeurogenesisStorageModeled<FIQ> {
    fn from(value: NMExclusivePowerNeurogenesisStorageQuanted<FIQ>) -> Self {
        Self::ExclusivePower(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMNeurogenesisStorageModeled<FIQ>> for NMExclusivePowerNeurogenesisStorageQuanted<FIQ> {
    type Error = NeuronModelEnumConversionError<NMKind>;

    fn try_from(value: NMNeurogenesisStorageModeled<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMNeurogenesisStorageModeled::ExclusivePower(v) => Ok(v),
            _ => Err(NeuronModelEnumConversionError::IncompatibleModel),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<NMExclusivePowerNeurogenesisStorageOrGeneratorQuanted<FIQ>> for NMNeurogenesisStorageOrGeneratorModeled<FIQ> {
    fn from(value: NMExclusivePowerNeurogenesisStorageOrGeneratorQuanted<FIQ>) -> Self {
        Self::ExclusivePower(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMNeurogenesisStorageOrGeneratorModeled<FIQ>> for NMExclusivePowerNeurogenesisStorageOrGeneratorQuanted<FIQ> {
    type Error = NeuronModelEnumConversionError<NMKind>;

    fn try_from(value: NMNeurogenesisStorageOrGeneratorModeled<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMNeurogenesisStorageOrGeneratorModeled::ExclusivePower(v) => Ok(v),
            _ => Err(NeuronModelEnumConversionError::IncompatibleModel),
        }
    }
}

impl<FIQ: FeagiIndexQuantization> From<NMExclusivePowerNeurogenesisValidatedGeneratorQuanted<FIQ>> for NMNeurogenesisValidatedGeneratorModeled<FIQ> {
    fn from(value: NMExclusivePowerNeurogenesisValidatedGeneratorQuanted<FIQ>) -> Self {
        Self::ExclusivePower(value)
    }
}

impl<FIQ: FeagiIndexQuantization> TryFrom<NMNeurogenesisValidatedGeneratorModeled<FIQ>> for NMExclusivePowerNeurogenesisValidatedGeneratorQuanted<FIQ> {
    type Error = NeuronModelEnumConversionError<NMKind>;

    fn try_from(value: NMNeurogenesisValidatedGeneratorModeled<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMNeurogenesisValidatedGeneratorModeled::ExclusivePower(v) => Ok(v),
            _ => Err(NeuronModelEnumConversionError::IncompatibleModel),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<NMExclusivePowerNeurogenesisValidatedStorageQuanted<FIQ>> for NMNeurogenesisValidatedStorageModeled<FIQ> {
    fn from(value: NMExclusivePowerNeurogenesisValidatedStorageQuanted<FIQ>) -> Self {
        Self::ExclusivePower(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMNeurogenesisValidatedStorageModeled<FIQ>> for NMExclusivePowerNeurogenesisValidatedStorageQuanted<FIQ> {
    type Error = NeuronModelEnumConversionError<NMKind>;

    fn try_from(value: NMNeurogenesisValidatedStorageModeled<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMNeurogenesisValidatedStorageModeled::ExclusivePower(v) => Ok(v),
            _ => Err(NeuronModelEnumConversionError::IncompatibleModel),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<NMExclusivePowerNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ>> for NMNeurogenesisValidatedStorageOrGeneratorModeled<FIQ> {
    fn from(value: NMExclusivePowerNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ>) -> Self {
        Self::ExclusivePower(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMNeurogenesisValidatedStorageOrGeneratorModeled<FIQ>> for NMExclusivePowerNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ> {
    type Error = NeuronModelEnumConversionError<NMKind>;

    fn try_from(value: NMNeurogenesisValidatedStorageOrGeneratorModeled<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMNeurogenesisValidatedStorageOrGeneratorModeled::ExclusivePower(v) => Ok(v),
            _ => Err(NeuronModelEnumConversionError::IncompatibleModel),
        }
    }
}

//endregion
