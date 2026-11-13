use crate::neuron_models::models::exclusive_death::{NMExclusiveDeathDataCorticalInternal, NMExclusiveDeathDataCorticalProperties, NMExclusiveDeathDataCorticalScratch, NMExclusiveDeathDataCorticalVariance, NMExclusiveDeathDataNeuronInternal, NMExclusiveDeathDataNeuronScratch, NMExclusiveDeathDataNeuronVariance, NMExclusiveDeathTrait};
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
/// Describes the quantization of the Exclusive Death Neuron Model
pub enum NMExclusiveDeathQuantKind {
    Float16,
    BFloat16,
    Float32,
    Float64,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum NMExclusiveDeathQuantedKind {
    Float16(NMExclusiveDeathQuantFloat16),
    BFloat16(NMExclusiveDeathQuantBFloat16),
    Float32(NMExclusiveDeathQuantFloat32),
    Float64(NMExclusiveDeathQuantFloat64)
}

impl NMExclusiveDeathQuantedKind {
    pub fn quant_kind(&self) -> NMExclusiveDeathQuantKind {
        match &self {
            NMExclusiveDeathQuantedKind::Float16(_) => NMExclusiveDeathQuantKind::Float16,
            NMExclusiveDeathQuantedKind::BFloat16(_) => NMExclusiveDeathQuantKind::BFloat16,
            NMExclusiveDeathQuantedKind::Float32(_) => NMExclusiveDeathQuantKind::Float32,
            NMExclusiveDeathQuantedKind::Float64(_) => NMExclusiveDeathQuantKind::Float64,
        }
    }

    pub fn from_nm_quant<NM: NMExclusiveDeathTrait>() -> NMExclusiveDeathQuantedKind {
        match NM::QUANT_KIND {
            NMExclusiveDeathQuantKind::Float16 => NMExclusiveDeathQuantedKind::Float16(NMExclusiveDeathQuantFloat16),
            NMExclusiveDeathQuantKind::BFloat16 => NMExclusiveDeathQuantedKind::BFloat16(NMExclusiveDeathQuantBFloat16),
            NMExclusiveDeathQuantKind::Float32 => NMExclusiveDeathQuantedKind::Float32(NMExclusiveDeathQuantFloat32),
            NMExclusiveDeathQuantKind::Float64 => NMExclusiveDeathQuantedKind::Float64(NMExclusiveDeathQuantFloat64),
        }
    }
}


//region Neurogenesis

/// The neurogenesis generator of the Exclusive Death Neuron Model, for a given quantization
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(bound = "")]
pub enum NMExclusiveDeathNeurogenesisGeneratorQuanted<FIQ: FeagiIndexQuantization> {
    Float16(NeurogenesisByGeneratorTypeOf<FIQ, NMExclusiveDeathQuantFloat16>),
    BFloat16(NeurogenesisByGeneratorTypeOf<FIQ, NMExclusiveDeathQuantBFloat16>),
    Float32(NeurogenesisByGeneratorTypeOf<FIQ, NMExclusiveDeathQuantFloat32>),
    Float64(NeurogenesisByGeneratorTypeOf<FIQ, NMExclusiveDeathQuantFloat64>),
}

impl<FIQ: FeagiIndexQuantization> NMExclusiveDeathNeurogenesisGeneratorQuanted<FIQ> {
    /// The quantization of the data held
    pub const fn quant_kind(&self) -> NMExclusiveDeathQuantKind {
        match self {
            Self::Float16(_) => NMExclusiveDeathQuantKind::Float16,
            Self::BFloat16(_) => NMExclusiveDeathQuantKind::BFloat16,
            Self::Float32(_) => NMExclusiveDeathQuantKind::Float32,
            Self::Float64(_) => NMExclusiveDeathQuantKind::Float64,
        }
    }
}

/// The stored neuron data of the Exclusive Death Neuron Model, for a given quantization
#[cfg(feature = "alloc")]
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(bound = "")]
pub enum NMExclusiveDeathNeurogenesisStorageQuanted<FIQ: FeagiIndexQuantization> {
    Float16(NeurogenesisFromStorageOf<FIQ, NMExclusiveDeathQuantFloat16>),
    BFloat16(NeurogenesisFromStorageOf<FIQ, NMExclusiveDeathQuantBFloat16>),
    Float32(NeurogenesisFromStorageOf<FIQ, NMExclusiveDeathQuantFloat32>),
    Float64(NeurogenesisFromStorageOf<FIQ, NMExclusiveDeathQuantFloat64>),
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> NMExclusiveDeathNeurogenesisStorageQuanted<FIQ> {
    /// The quantization of the data held
    pub const fn quant_kind(&self) -> NMExclusiveDeathQuantKind {
        match self {
            Self::Float16(_) => NMExclusiveDeathQuantKind::Float16,
            Self::BFloat16(_) => NMExclusiveDeathQuantKind::BFloat16,
            Self::Float32(_) => NMExclusiveDeathQuantKind::Float32,
            Self::Float64(_) => NMExclusiveDeathQuantKind::Float64,
        }
    }
}

/// The stored neuron data or the generator of the Exclusive Death Neuron Model, for a given quantization
#[cfg(feature = "alloc")]
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(bound = "")]
pub enum NMExclusiveDeathNeurogenesisStorageOrGeneratorQuanted<FIQ: FeagiIndexQuantization> {
    Float16(NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMExclusiveDeathQuantFloat16>),
    BFloat16(NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMExclusiveDeathQuantBFloat16>),
    Float32(NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMExclusiveDeathQuantFloat32>),
    Float64(NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMExclusiveDeathQuantFloat64>),
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> NMExclusiveDeathNeurogenesisStorageOrGeneratorQuanted<FIQ> {
    /// The quantization of the data held
    pub const fn quant_kind(&self) -> NMExclusiveDeathQuantKind {
        match self {
            Self::Float16(_) => NMExclusiveDeathQuantKind::Float16,
            Self::BFloat16(_) => NMExclusiveDeathQuantKind::BFloat16,
            Self::Float32(_) => NMExclusiveDeathQuantKind::Float32,
            Self::Float64(_) => NMExclusiveDeathQuantKind::Float64,
        }
    }
}

/// The validated neurogenesis generator of the Exclusive Death Neuron Model, for a given quantization
#[derive(Debug, Clone)]
pub enum NMExclusiveDeathNeurogenesisValidatedGeneratorQuanted<FIQ: FeagiIndexQuantization> {
    Float16(ValidatedNeurogenesisGenerator<FIQ, NMExclusiveDeathQuantFloat16>),
    BFloat16(ValidatedNeurogenesisGenerator<FIQ, NMExclusiveDeathQuantBFloat16>),
    Float32(ValidatedNeurogenesisGenerator<FIQ, NMExclusiveDeathQuantFloat32>),
    Float64(ValidatedNeurogenesisGenerator<FIQ, NMExclusiveDeathQuantFloat64>),
}

impl<FIQ: FeagiIndexQuantization> NMExclusiveDeathNeurogenesisValidatedGeneratorQuanted<FIQ> {
    /// The quantization of the data held
    pub const fn quant_kind(&self) -> NMExclusiveDeathQuantKind {
        match self {
            Self::Float16(_) => NMExclusiveDeathQuantKind::Float16,
            Self::BFloat16(_) => NMExclusiveDeathQuantKind::BFloat16,
            Self::Float32(_) => NMExclusiveDeathQuantKind::Float32,
            Self::Float64(_) => NMExclusiveDeathQuantKind::Float64,
        }
    }
}

/// The validated stored neuron data of the Exclusive Death Neuron Model, for a given quantization
#[cfg(feature = "alloc")]
#[derive(Debug, Clone)]
pub enum NMExclusiveDeathNeurogenesisValidatedStorageQuanted<FIQ: FeagiIndexQuantization> {
    Float16(ValidatedNeurogenesisStorage<FIQ, NMExclusiveDeathQuantFloat16>),
    BFloat16(ValidatedNeurogenesisStorage<FIQ, NMExclusiveDeathQuantBFloat16>),
    Float32(ValidatedNeurogenesisStorage<FIQ, NMExclusiveDeathQuantFloat32>),
    Float64(ValidatedNeurogenesisStorage<FIQ, NMExclusiveDeathQuantFloat64>),
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> NMExclusiveDeathNeurogenesisValidatedStorageQuanted<FIQ> {
    /// The quantization of the data held
    pub const fn quant_kind(&self) -> NMExclusiveDeathQuantKind {
        match self {
            Self::Float16(_) => NMExclusiveDeathQuantKind::Float16,
            Self::BFloat16(_) => NMExclusiveDeathQuantKind::BFloat16,
            Self::Float32(_) => NMExclusiveDeathQuantKind::Float32,
            Self::Float64(_) => NMExclusiveDeathQuantKind::Float64,
        }
    }
}

/// The validated stored neuron data or generator of the Exclusive Death Neuron Model, for a given quantization
#[cfg(feature = "alloc")]
#[derive(Debug, Clone)]
pub enum NMExclusiveDeathNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ: FeagiIndexQuantization> {
    Float16(ValidatedNeurogenesisStorageOrGenerator<FIQ, NMExclusiveDeathQuantFloat16>),
    BFloat16(ValidatedNeurogenesisStorageOrGenerator<FIQ, NMExclusiveDeathQuantBFloat16>),
    Float32(ValidatedNeurogenesisStorageOrGenerator<FIQ, NMExclusiveDeathQuantFloat32>),
    Float64(ValidatedNeurogenesisStorageOrGenerator<FIQ, NMExclusiveDeathQuantFloat64>),
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> NMExclusiveDeathNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ> {
    /// The quantization of the data held
    pub const fn quant_kind(&self) -> NMExclusiveDeathQuantKind {
        match self {
            Self::Float16(_) => NMExclusiveDeathQuantKind::Float16,
            Self::BFloat16(_) => NMExclusiveDeathQuantKind::BFloat16,
            Self::Float32(_) => NMExclusiveDeathQuantKind::Float32,
            Self::Float64(_) => NMExclusiveDeathQuantKind::Float64,
        }
    }
}

//endregion

//endregion

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMExclusiveDeathQuantFloat16;

impl NMExclusiveDeathTrait for NMExclusiveDeathQuantFloat16 {
}

impl NeuronModel for NMExclusiveDeathQuantFloat16 {
    type NeuronLayoutTag = NeuronLayoutTagVoxel;

    type NeurogenesisScratchVarianceDefaults<FIQ: FeagiIndexQuantization> =
        super::neurogenesis::ExclusiveDeathNeurogenesisDefaults<FIQ, Self>;

    type NeurogenesisByGenerator<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorUniformDefault<FIQ, Self>;

    type CorticalDataProperties = NMExclusiveDeathDataCorticalProperties<Self>;
    type CorticalDataInternal = NMExclusiveDeathDataCorticalInternal<Self>;
    type CorticalDataScratch = NMExclusiveDeathDataCorticalScratch<Self>;
    type CorticalDataVariance = NMExclusiveDeathDataCorticalVariance<Self>;
    type NeuronDataInternal = NMExclusiveDeathDataNeuronInternal<Self>;
    type NeuronDataScratch = NMExclusiveDeathDataNeuronScratch<Self>;
    type NeuronDataVariance = NMExclusiveDeathDataNeuronVariance<Self>;

    type QuantKind = NMExclusiveDeathQuantKind;
    type QuantedKind = NMExclusiveDeathQuantedKind;
    const QUANT_KIND: Self::QuantKind = NMExclusiveDeathQuantKind::Float16;
    const QUANTED_KIND: Self::QuantedKind = NMExclusiveDeathQuantedKind::Float16(Self);
}

impl MembranePotentialQuantization for NMExclusiveDeathQuantFloat16
{
    type MembranePotentialQuant = f16;
}

impl From<NMExclusiveDeathQuantFloat16> for NMExclusiveDeathQuantedKind {
    fn from(value: NMExclusiveDeathQuantFloat16) -> Self {
        NMExclusiveDeathQuantedKind::Float16(value)
    }
}

impl TryFrom<NMExclusiveDeathQuantedKind> for NMExclusiveDeathQuantFloat16 {
    type Error = NeuronModelEnumConversionError<NMExclusiveDeathQuantKind>;

    fn try_from(value: NMExclusiveDeathQuantedKind) -> Result<Self, Self::Error> {
        let actual = value.quant_kind();
        match value {
            NMExclusiveDeathQuantedKind::Float16(s) => {Ok(s)},
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMExclusiveDeathQuantKind::Float16,
                    actual,
                })
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMExclusiveDeathQuantBFloat16;

impl NMExclusiveDeathTrait for NMExclusiveDeathQuantBFloat16 {
}

impl NeuronModel for NMExclusiveDeathQuantBFloat16 {
    type NeuronLayoutTag = NeuronLayoutTagVoxel;

    type NeurogenesisScratchVarianceDefaults<FIQ: FeagiIndexQuantization> =
        super::neurogenesis::ExclusiveDeathNeurogenesisDefaults<FIQ, Self>;

    type NeurogenesisByGenerator<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorUniformDefault<FIQ, Self>;

    type CorticalDataProperties = NMExclusiveDeathDataCorticalProperties<Self>;
    type CorticalDataInternal = NMExclusiveDeathDataCorticalInternal<Self>;
    type CorticalDataScratch = NMExclusiveDeathDataCorticalScratch<Self>;
    type CorticalDataVariance = NMExclusiveDeathDataCorticalVariance<Self>;
    type NeuronDataInternal = NMExclusiveDeathDataNeuronInternal<Self>;
    type NeuronDataScratch = NMExclusiveDeathDataNeuronScratch<Self>;
    type NeuronDataVariance = NMExclusiveDeathDataNeuronVariance<Self>;

    type QuantKind = NMExclusiveDeathQuantKind;
    type QuantedKind = NMExclusiveDeathQuantedKind;
    const QUANT_KIND: Self::QuantKind = NMExclusiveDeathQuantKind::BFloat16;
    const QUANTED_KIND: Self::QuantedKind = NMExclusiveDeathQuantedKind::BFloat16(Self);
}

impl MembranePotentialQuantization for NMExclusiveDeathQuantBFloat16
{
    type MembranePotentialQuant = bf16;
}

impl From<NMExclusiveDeathQuantBFloat16> for NMExclusiveDeathQuantedKind {
    fn from(value: NMExclusiveDeathQuantBFloat16) -> Self {
        NMExclusiveDeathQuantedKind::BFloat16(value)
    }
}

impl TryFrom<NMExclusiveDeathQuantedKind> for NMExclusiveDeathQuantBFloat16 {
    type Error = NeuronModelEnumConversionError<NMExclusiveDeathQuantKind>;

    fn try_from(value: NMExclusiveDeathQuantedKind) -> Result<Self, Self::Error> {
        let actual = value.quant_kind();
        match value {
            NMExclusiveDeathQuantedKind::BFloat16(s) => {Ok(s)},
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMExclusiveDeathQuantKind::BFloat16,
                    actual,
                })
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMExclusiveDeathQuantFloat32;

impl NMExclusiveDeathTrait for NMExclusiveDeathQuantFloat32 {
}

impl NeuronModel for NMExclusiveDeathQuantFloat32 {
    type NeuronLayoutTag = NeuronLayoutTagVoxel;

    type NeurogenesisScratchVarianceDefaults<FIQ: FeagiIndexQuantization> =
        super::neurogenesis::ExclusiveDeathNeurogenesisDefaults<FIQ, Self>;

    type NeurogenesisByGenerator<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorUniformDefault<FIQ, Self>;

    type CorticalDataProperties = NMExclusiveDeathDataCorticalProperties<Self>;
    type CorticalDataInternal = NMExclusiveDeathDataCorticalInternal<Self>;
    type CorticalDataScratch = NMExclusiveDeathDataCorticalScratch<Self>;
    type CorticalDataVariance = NMExclusiveDeathDataCorticalVariance<Self>;
    type NeuronDataInternal = NMExclusiveDeathDataNeuronInternal<Self>;
    type NeuronDataScratch = NMExclusiveDeathDataNeuronScratch<Self>;
    type NeuronDataVariance = NMExclusiveDeathDataNeuronVariance<Self>;

    type QuantKind = NMExclusiveDeathQuantKind;
    type QuantedKind = NMExclusiveDeathQuantedKind;
    const QUANT_KIND: Self::QuantKind = NMExclusiveDeathQuantKind::Float32;
    const QUANTED_KIND: Self::QuantedKind = NMExclusiveDeathQuantedKind::Float32(Self);
}

impl MembranePotentialQuantization for NMExclusiveDeathQuantFloat32
{
    type MembranePotentialQuant = f32;
}

impl From<NMExclusiveDeathQuantFloat32> for NMExclusiveDeathQuantedKind {
    fn from(value: NMExclusiveDeathQuantFloat32) -> Self {
        NMExclusiveDeathQuantedKind::Float32(value)
    }
}

impl TryFrom<NMExclusiveDeathQuantedKind> for NMExclusiveDeathQuantFloat32 {
    type Error = NeuronModelEnumConversionError<NMExclusiveDeathQuantKind>;

    fn try_from(value: NMExclusiveDeathQuantedKind) -> Result<Self, Self::Error> {
        let actual = value.quant_kind();
        match value {
            NMExclusiveDeathQuantedKind::Float32(s) => {Ok(s)},
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMExclusiveDeathQuantKind::Float32,
                    actual,
                })
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMExclusiveDeathQuantFloat64;

impl NMExclusiveDeathTrait for NMExclusiveDeathQuantFloat64 {
}

impl NeuronModel for NMExclusiveDeathQuantFloat64 {
    type NeuronLayoutTag = NeuronLayoutTagVoxel;

    type NeurogenesisScratchVarianceDefaults<FIQ: FeagiIndexQuantization> =
        super::neurogenesis::ExclusiveDeathNeurogenesisDefaults<FIQ, Self>;

    type NeurogenesisByGenerator<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorUniformDefault<FIQ, Self>;

    type CorticalDataProperties = NMExclusiveDeathDataCorticalProperties<Self>;
    type CorticalDataInternal = NMExclusiveDeathDataCorticalInternal<Self>;
    type CorticalDataScratch = NMExclusiveDeathDataCorticalScratch<Self>;
    type CorticalDataVariance = NMExclusiveDeathDataCorticalVariance<Self>;
    type NeuronDataInternal = NMExclusiveDeathDataNeuronInternal<Self>;
    type NeuronDataScratch = NMExclusiveDeathDataNeuronScratch<Self>;
    type NeuronDataVariance = NMExclusiveDeathDataNeuronVariance<Self>;

    type QuantKind = NMExclusiveDeathQuantKind;
    type QuantedKind = NMExclusiveDeathQuantedKind;
    const QUANT_KIND: Self::QuantKind = NMExclusiveDeathQuantKind::Float64;
    const QUANTED_KIND: Self::QuantedKind = NMExclusiveDeathQuantedKind::Float64(Self);
}

impl MembranePotentialQuantization for NMExclusiveDeathQuantFloat64
{
    type MembranePotentialQuant = f64;
}

impl From<NMExclusiveDeathQuantFloat64> for NMExclusiveDeathQuantedKind {
    fn from(value: NMExclusiveDeathQuantFloat64) -> Self {
        NMExclusiveDeathQuantedKind::Float64(value)
    }
}

impl TryFrom<NMExclusiveDeathQuantedKind> for NMExclusiveDeathQuantFloat64 {
    type Error = NeuronModelEnumConversionError<NMExclusiveDeathQuantKind>;

    fn try_from(value: NMExclusiveDeathQuantedKind) -> Result<Self, Self::Error> {
        let actual = value.quant_kind();
        match value {
            NMExclusiveDeathQuantedKind::Float64(s) => {Ok(s)},
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMExclusiveDeathQuantKind::Float64,
                    actual,
                })
            }
        }
    }
}

//region Model Level Conversions

impl From<NMExclusiveDeathQuantKind> for NMKind {
    fn from(value: NMExclusiveDeathQuantKind) -> Self {
        NMKind::ExclusiveDeath(value)
    }
}

impl TryFrom<NMKind> for NMExclusiveDeathQuantKind {
    type Error = NeuronModelEnumConversionError<NMKind>;

    fn try_from(value: NMKind) -> Result<Self, Self::Error> {
        match value {
            NMKind::ExclusiveDeath(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleModel)
            }
        }
    }
}

impl From<NMExclusiveDeathQuantedKind> for NMQuantedKind {
    fn from(value: NMExclusiveDeathQuantedKind) -> Self {
        NMQuantedKind::ExclusiveDeath(value)
    }
}

impl TryFrom<NMQuantedKind> for NMExclusiveDeathQuantedKind {
    type Error = NeuronModelEnumConversionError<NMKind>;

    fn try_from(value: NMQuantedKind) -> Result<Self, Self::Error> {
        match value {
            NMQuantedKind::ExclusiveDeath(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleModel)
            }
        }
    }
}

impl<FIQ: FeagiIndexQuantization> From<NMExclusiveDeathNeurogenesisGeneratorQuanted<FIQ>> for NMNeurogenesisGeneratorModeled<FIQ> {
    fn from(value: NMExclusiveDeathNeurogenesisGeneratorQuanted<FIQ>) -> Self {
        Self::ExclusiveDeath(value)
    }
}

impl<FIQ: FeagiIndexQuantization> TryFrom<NMNeurogenesisGeneratorModeled<FIQ>> for NMExclusiveDeathNeurogenesisGeneratorQuanted<FIQ> {
    type Error = NeuronModelEnumConversionError<NMKind>;

    fn try_from(value: NMNeurogenesisGeneratorModeled<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMNeurogenesisGeneratorModeled::ExclusiveDeath(v) => Ok(v),
            _ => Err(NeuronModelEnumConversionError::IncompatibleModel),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<NMExclusiveDeathNeurogenesisStorageQuanted<FIQ>> for NMNeurogenesisStorageModeled<FIQ> {
    fn from(value: NMExclusiveDeathNeurogenesisStorageQuanted<FIQ>) -> Self {
        Self::ExclusiveDeath(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMNeurogenesisStorageModeled<FIQ>> for NMExclusiveDeathNeurogenesisStorageQuanted<FIQ> {
    type Error = NeuronModelEnumConversionError<NMKind>;

    fn try_from(value: NMNeurogenesisStorageModeled<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMNeurogenesisStorageModeled::ExclusiveDeath(v) => Ok(v),
            _ => Err(NeuronModelEnumConversionError::IncompatibleModel),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<NMExclusiveDeathNeurogenesisStorageOrGeneratorQuanted<FIQ>> for NMNeurogenesisStorageOrGeneratorModeled<FIQ> {
    fn from(value: NMExclusiveDeathNeurogenesisStorageOrGeneratorQuanted<FIQ>) -> Self {
        Self::ExclusiveDeath(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMNeurogenesisStorageOrGeneratorModeled<FIQ>> for NMExclusiveDeathNeurogenesisStorageOrGeneratorQuanted<FIQ> {
    type Error = NeuronModelEnumConversionError<NMKind>;

    fn try_from(value: NMNeurogenesisStorageOrGeneratorModeled<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMNeurogenesisStorageOrGeneratorModeled::ExclusiveDeath(v) => Ok(v),
            _ => Err(NeuronModelEnumConversionError::IncompatibleModel),
        }
    }
}

impl<FIQ: FeagiIndexQuantization> From<NMExclusiveDeathNeurogenesisValidatedGeneratorQuanted<FIQ>> for NMNeurogenesisValidatedGeneratorModeled<FIQ> {
    fn from(value: NMExclusiveDeathNeurogenesisValidatedGeneratorQuanted<FIQ>) -> Self {
        Self::ExclusiveDeath(value)
    }
}

impl<FIQ: FeagiIndexQuantization> TryFrom<NMNeurogenesisValidatedGeneratorModeled<FIQ>> for NMExclusiveDeathNeurogenesisValidatedGeneratorQuanted<FIQ> {
    type Error = NeuronModelEnumConversionError<NMKind>;

    fn try_from(value: NMNeurogenesisValidatedGeneratorModeled<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMNeurogenesisValidatedGeneratorModeled::ExclusiveDeath(v) => Ok(v),
            _ => Err(NeuronModelEnumConversionError::IncompatibleModel),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<NMExclusiveDeathNeurogenesisValidatedStorageQuanted<FIQ>> for NMNeurogenesisValidatedStorageModeled<FIQ> {
    fn from(value: NMExclusiveDeathNeurogenesisValidatedStorageQuanted<FIQ>) -> Self {
        Self::ExclusiveDeath(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMNeurogenesisValidatedStorageModeled<FIQ>> for NMExclusiveDeathNeurogenesisValidatedStorageQuanted<FIQ> {
    type Error = NeuronModelEnumConversionError<NMKind>;

    fn try_from(value: NMNeurogenesisValidatedStorageModeled<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMNeurogenesisValidatedStorageModeled::ExclusiveDeath(v) => Ok(v),
            _ => Err(NeuronModelEnumConversionError::IncompatibleModel),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<NMExclusiveDeathNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ>> for NMNeurogenesisValidatedStorageOrGeneratorModeled<FIQ> {
    fn from(value: NMExclusiveDeathNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ>) -> Self {
        Self::ExclusiveDeath(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMNeurogenesisValidatedStorageOrGeneratorModeled<FIQ>> for NMExclusiveDeathNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ> {
    type Error = NeuronModelEnumConversionError<NMKind>;

    fn try_from(value: NMNeurogenesisValidatedStorageOrGeneratorModeled<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMNeurogenesisValidatedStorageOrGeneratorModeled::ExclusiveDeath(v) => Ok(v),
            _ => Err(NeuronModelEnumConversionError::IncompatibleModel),
        }
    }
}

//endregion
