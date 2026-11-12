use crate::neuron_models::models::foundation_power::{NMFoundationPowerDataCorticalInternal, NMFoundationPowerDataCorticalProperties, NMFoundationPowerDataCorticalScratch, NMFoundationPowerDataCorticalVariance, NMFoundationPowerDataNeuronInternal, NMFoundationPowerDataNeuronScratch, NMFoundationPowerDataNeuronVariance, NMFoundationPowerTrait};
use crate::neuron_models::models::kind::{NMKind, NMModelKind, NMQuantedKind};
use crate::neuron_models::models::{NMNeurogenesisGeneratorModeled, NMNeurogenesisGeneratorPrecursorModeled, NMNeurogenesisStorageModeled, NMNeurogenesisStorageOrGeneratorModeled, NeuronModelEnumConversionError};
use crate::neuron_models::models::{NeurogenesisByGeneratorPrecursorTypeOf, NeurogenesisByGeneratorTypeOf, NeurogenesisFromStorageOf, NeurogenesisFromStorageOrByGeneratorOf};
use crate::neuron_models::neurogenesis_generator_uniform_defaults::{NeurogenesisGeneratorPrecursorUniformDefault, NeurogenesisGeneratorUniformDefault};
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::voxel::NeuronLayoutTagVoxel;
use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome_neural::neural_basis::quantization_levels::{FeagiIndexQuantization, MembranePotentialQuantization};
use half::{bf16, f16};
use serde::{Deserialize, Serialize};
//region enums

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
/// Describes the quantization of the Foundation Power Neuron Model
pub enum NMFoundationPowerQuantKind {
    Float16,
    BFloat16,
    Float32,
    Float64,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum NMFoundationPowerQuantedKind {
    Float16(NMFoundationPowerQuantFloat16),
    BFloat16(NMFoundationPowerQuantBFloat16),
    Float32(NMFoundationPowerQuantFloat32),
    Float64(NMFoundationPowerQuantFloat64)
}

impl NMFoundationPowerQuantedKind {
    pub fn quant_kind(&self) -> NMFoundationPowerQuantKind {
        match &self {
            NMFoundationPowerQuantedKind::Float16(_) => NMFoundationPowerQuantKind::Float16,
            NMFoundationPowerQuantedKind::BFloat16(_) => NMFoundationPowerQuantKind::BFloat16,
            NMFoundationPowerQuantedKind::Float32(_) => NMFoundationPowerQuantKind::Float32,
            NMFoundationPowerQuantedKind::Float64(_) => NMFoundationPowerQuantKind::Float64,
        }
    }

    pub fn from_nm_quant<NM: NMFoundationPowerTrait>() -> NMFoundationPowerQuantedKind {
        match NM::QUANT_KIND {
            NMFoundationPowerQuantKind::Float16 => NMFoundationPowerQuantedKind::Float16(NMFoundationPowerQuantFloat16),
            NMFoundationPowerQuantKind::BFloat16 => NMFoundationPowerQuantedKind::BFloat16(NMFoundationPowerQuantBFloat16),
            NMFoundationPowerQuantKind::Float32 => NMFoundationPowerQuantedKind::Float32(NMFoundationPowerQuantFloat32),
            NMFoundationPowerQuantKind::Float64 => NMFoundationPowerQuantedKind::Float64(NMFoundationPowerQuantFloat64),
        }
    }
}


//region Generators

/// The data needed to create the generator of the Foundation Power Neuron Model, for a given quantization
#[derive(Debug, Clone)]
pub enum NMFoundationPowerNeurogenesisGeneratorPrecursorQuanted<FIQ: FeagiIndexQuantization> {
    Float16(NeurogenesisByGeneratorPrecursorTypeOf<FIQ, NMFoundationPowerQuantFloat16>),
    BFloat16(NeurogenesisByGeneratorPrecursorTypeOf<FIQ, NMFoundationPowerQuantBFloat16>),
    Float32(NeurogenesisByGeneratorPrecursorTypeOf<FIQ, NMFoundationPowerQuantFloat32>),
    Float64(NeurogenesisByGeneratorPrecursorTypeOf<FIQ, NMFoundationPowerQuantFloat64>)
}

impl<FIQ: FeagiIndexQuantization> NMFoundationPowerNeurogenesisGeneratorPrecursorQuanted<FIQ> {
    /// The quantization of the precursor held
    pub const fn quant_kind(&self) -> NMFoundationPowerQuantKind {
        match self {
            Self::Float16(_) => NMFoundationPowerQuantKind::Float16,
            Self::BFloat16(_) => NMFoundationPowerQuantKind::BFloat16,
            Self::Float32(_) => NMFoundationPowerQuantKind::Float32,
            Self::Float64(_) => NMFoundationPowerQuantKind::Float64,
        }
    }
}

/// Allows Neurogenesis of the Foundation Power Neuron Model by its generator for a given quantization
#[derive(Debug, Clone, Serialize, Deserialize)]

#[serde(bound = "")]
pub enum NMFoundationPowerNeurogenesisGeneratorQuanted<FIQ: FeagiIndexQuantization> {
    Float16(NeurogenesisByGeneratorTypeOf<FIQ, NMFoundationPowerQuantFloat16>),
    BFloat16(NeurogenesisByGeneratorTypeOf<FIQ, NMFoundationPowerQuantBFloat16>),
    Float32(NeurogenesisByGeneratorTypeOf<FIQ, NMFoundationPowerQuantFloat32>),
    Float64(NeurogenesisByGeneratorTypeOf<FIQ, NMFoundationPowerQuantFloat64>)
}

impl<FIQ: FeagiIndexQuantization> NMFoundationPowerNeurogenesisGeneratorQuanted<FIQ> {
    /// The quantization of the generator held
    pub const fn quant_kind(&self) -> NMFoundationPowerQuantKind {
        match self {
            Self::Float16(_) => NMFoundationPowerQuantKind::Float16,
            Self::BFloat16(_) => NMFoundationPowerQuantKind::BFloat16,
            Self::Float32(_) => NMFoundationPowerQuantKind::Float32,
            Self::Float64(_) => NMFoundationPowerQuantKind::Float64,
        }
    }
}

/// Allows Neurogenesis of the Foundation Power Neuron Model from its stored data for a given quantization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum NMFoundationPowerNeurogenesisStorageQuanted<FIQ: FeagiIndexQuantization> {
    Float16(NeurogenesisFromStorageOf<FIQ, NMFoundationPowerQuantFloat16>),
    BFloat16(NeurogenesisFromStorageOf<FIQ, NMFoundationPowerQuantBFloat16>),
    Float32(NeurogenesisFromStorageOf<FIQ, NMFoundationPowerQuantFloat32>),
    Float64(NeurogenesisFromStorageOf<FIQ, NMFoundationPowerQuantFloat64>)
}

impl<FIQ: FeagiIndexQuantization> NMFoundationPowerNeurogenesisStorageQuanted<FIQ> {
    /// The quantization of the storage held
    pub const fn quant_kind(&self) -> NMFoundationPowerQuantKind {
        match self {
            Self::Float16(_) => NMFoundationPowerQuantKind::Float16,
            Self::BFloat16(_) => NMFoundationPowerQuantKind::BFloat16,
            Self::Float32(_) => NMFoundationPowerQuantKind::Float32,
            Self::Float64(_) => NMFoundationPowerQuantKind::Float64,
        }
    }
}

/// Allows Neurogenesis of the Foundation Power Neuron Model from either its stored data or by its
/// generator, for a given quantization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum NMFoundationPowerNeurogenesisStorageOrGeneratorQuanted<FIQ: FeagiIndexQuantization> {
    Float16(NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMFoundationPowerQuantFloat16>),
    BFloat16(NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMFoundationPowerQuantBFloat16>),
    Float32(NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMFoundationPowerQuantFloat32>),
    Float64(NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMFoundationPowerQuantFloat64>)
}

impl<FIQ: FeagiIndexQuantization> NMFoundationPowerNeurogenesisStorageOrGeneratorQuanted<FIQ> {
    /// The quantization of the storage or generator held
    pub const fn quant_kind(&self) -> NMFoundationPowerQuantKind {
        match self {
            Self::Float16(_) => NMFoundationPowerQuantKind::Float16,
            Self::BFloat16(_) => NMFoundationPowerQuantKind::BFloat16,
            Self::Float32(_) => NMFoundationPowerQuantKind::Float32,
            Self::Float64(_) => NMFoundationPowerQuantKind::Float64,
        }
    }
}


//endregion

//endregion

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMFoundationPowerQuantFloat16;

impl NMFoundationPowerTrait for NMFoundationPowerQuantFloat16 {
}

impl NeuronModel for NMFoundationPowerQuantFloat16 {
    type NeuronLayoutTag = NeuronLayoutTagVoxel;

    type NeurogenesisScratchVarianceGeneratorType<FIQ: FeagiIndexQuantization> =
        super::neurogenesis::FoundationPowerNeurogenesisDefaults<FIQ, Self>;

    type NeurogenesisGenerator<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorUniformDefault<FIQ, Self>;
    type NeurogenesisGeneratorPrecursor<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorPrecursorUniformDefault<FIQ, Self>;

    type CorticalDataProperties = NMFoundationPowerDataCorticalProperties<Self>;
    type CorticalDataInternal = NMFoundationPowerDataCorticalInternal<Self>;
    type CorticalDataScratch = NMFoundationPowerDataCorticalScratch<Self>;
    type CorticalDataVariance = NMFoundationPowerDataCorticalVariance<Self>;
    type NeuronDataInternal = NMFoundationPowerDataNeuronInternal<Self>;
    type NeuronDataScratch = NMFoundationPowerDataNeuronScratch<Self>;
    type NeuronDataVariance = NMFoundationPowerDataNeuronVariance<Self>;

    type QuantKind = NMFoundationPowerQuantKind;
    type QuantedKind = NMFoundationPowerQuantedKind;
    const QUANT_KIND: Self::QuantKind = NMFoundationPowerQuantKind::Float16;
    const QUANTED_KIND: Self::QuantedKind = NMFoundationPowerQuantedKind::Float16(Self);
}

impl MembranePotentialQuantization for NMFoundationPowerQuantFloat16
{
    type MembranePotentialQuant = f16;
}

impl From<NMFoundationPowerQuantFloat16> for NMFoundationPowerQuantedKind {
    fn from(value: NMFoundationPowerQuantFloat16) -> Self {
        NMFoundationPowerQuantedKind::Float16(value)
    }
}

impl TryFrom<NMFoundationPowerQuantedKind> for NMFoundationPowerQuantFloat16 {
    type Error = NeuronModelEnumConversionError<NMFoundationPowerQuantKind>;

    fn try_from(value: NMFoundationPowerQuantedKind) -> Result<Self, Self::Error> {
        let actual = value.quant_kind();
        match value {
            NMFoundationPowerQuantedKind::Float16(s) => {Ok(s)},
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMFoundationPowerQuantKind::Float16,
                    actual,
                })
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMFoundationPowerQuantBFloat16;

impl NMFoundationPowerTrait for NMFoundationPowerQuantBFloat16 {
}

impl NeuronModel for NMFoundationPowerQuantBFloat16 {
    type NeuronLayoutTag = NeuronLayoutTagVoxel;

    type NeurogenesisScratchVarianceGeneratorType<FIQ: FeagiIndexQuantization> =
        super::neurogenesis::FoundationPowerNeurogenesisDefaults<FIQ, Self>;

    type NeurogenesisGenerator<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorUniformDefault<FIQ, Self>;
    type NeurogenesisGeneratorPrecursor<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorPrecursorUniformDefault<FIQ, Self>;

    type CorticalDataProperties = NMFoundationPowerDataCorticalProperties<Self>;
    type CorticalDataInternal = NMFoundationPowerDataCorticalInternal<Self>;
    type CorticalDataScratch = NMFoundationPowerDataCorticalScratch<Self>;
    type CorticalDataVariance = NMFoundationPowerDataCorticalVariance<Self>;
    type NeuronDataInternal = NMFoundationPowerDataNeuronInternal<Self>;
    type NeuronDataScratch = NMFoundationPowerDataNeuronScratch<Self>;
    type NeuronDataVariance = NMFoundationPowerDataNeuronVariance<Self>;

    type QuantKind = NMFoundationPowerQuantKind;
    type QuantedKind = NMFoundationPowerQuantedKind;
    const QUANT_KIND: Self::QuantKind = NMFoundationPowerQuantKind::BFloat16;
    const QUANTED_KIND: Self::QuantedKind = NMFoundationPowerQuantedKind::BFloat16(Self);
}

impl MembranePotentialQuantization for NMFoundationPowerQuantBFloat16
{
    type MembranePotentialQuant = bf16;
}

impl From<NMFoundationPowerQuantBFloat16> for NMFoundationPowerQuantedKind {
    fn from(value: NMFoundationPowerQuantBFloat16) -> Self {
        NMFoundationPowerQuantedKind::BFloat16(value)
    }
}

impl TryFrom<NMFoundationPowerQuantedKind> for NMFoundationPowerQuantBFloat16 {
    type Error = NeuronModelEnumConversionError<NMFoundationPowerQuantKind>;

    fn try_from(value: NMFoundationPowerQuantedKind) -> Result<Self, Self::Error> {
        let actual = value.quant_kind();
        match value {
            NMFoundationPowerQuantedKind::BFloat16(s) => {Ok(s)},
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMFoundationPowerQuantKind::BFloat16,
                    actual,
                })
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMFoundationPowerQuantFloat32;

impl NMFoundationPowerTrait for NMFoundationPowerQuantFloat32 {
}

impl NeuronModel for NMFoundationPowerQuantFloat32 {
    type NeuronLayoutTag = NeuronLayoutTagVoxel;

    type NeurogenesisScratchVarianceGeneratorType<FIQ: FeagiIndexQuantization> =
        super::neurogenesis::FoundationPowerNeurogenesisDefaults<FIQ, Self>;

    type NeurogenesisGenerator<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorUniformDefault<FIQ, Self>;
    type NeurogenesisGeneratorPrecursor<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorPrecursorUniformDefault<FIQ, Self>;

    type CorticalDataProperties = NMFoundationPowerDataCorticalProperties<Self>;
    type CorticalDataInternal = NMFoundationPowerDataCorticalInternal<Self>;
    type CorticalDataScratch = NMFoundationPowerDataCorticalScratch<Self>;
    type CorticalDataVariance = NMFoundationPowerDataCorticalVariance<Self>;
    type NeuronDataInternal = NMFoundationPowerDataNeuronInternal<Self>;
    type NeuronDataScratch = NMFoundationPowerDataNeuronScratch<Self>;
    type NeuronDataVariance = NMFoundationPowerDataNeuronVariance<Self>;

    type QuantKind = NMFoundationPowerQuantKind;
    type QuantedKind = NMFoundationPowerQuantedKind;
    const QUANT_KIND: Self::QuantKind = NMFoundationPowerQuantKind::Float32;
    const QUANTED_KIND: Self::QuantedKind = NMFoundationPowerQuantedKind::Float32(Self);
}

impl MembranePotentialQuantization for NMFoundationPowerQuantFloat32
{
    type MembranePotentialQuant = f32;
}

impl From<NMFoundationPowerQuantFloat32> for NMFoundationPowerQuantedKind {
    fn from(value: NMFoundationPowerQuantFloat32) -> Self {
        NMFoundationPowerQuantedKind::Float32(value)
    }
}

impl TryFrom<NMFoundationPowerQuantedKind> for NMFoundationPowerQuantFloat32 {
    type Error = NeuronModelEnumConversionError<NMFoundationPowerQuantKind>;

    fn try_from(value: NMFoundationPowerQuantedKind) -> Result<Self, Self::Error> {
        let actual = value.quant_kind();
        match value {
            NMFoundationPowerQuantedKind::Float32(s) => {Ok(s)},
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMFoundationPowerQuantKind::Float32,
                    actual,
                })
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMFoundationPowerQuantFloat64;

impl NMFoundationPowerTrait for NMFoundationPowerQuantFloat64 {
}

impl NeuronModel for NMFoundationPowerQuantFloat64 {
    type NeuronLayoutTag = NeuronLayoutTagVoxel;

    type NeurogenesisScratchVarianceGeneratorType<FIQ: FeagiIndexQuantization> =
        super::neurogenesis::FoundationPowerNeurogenesisDefaults<FIQ, Self>;

    type NeurogenesisGenerator<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorUniformDefault<FIQ, Self>;
    type NeurogenesisGeneratorPrecursor<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorPrecursorUniformDefault<FIQ, Self>;

    type CorticalDataProperties = NMFoundationPowerDataCorticalProperties<Self>;
    type CorticalDataInternal = NMFoundationPowerDataCorticalInternal<Self>;
    type CorticalDataScratch = NMFoundationPowerDataCorticalScratch<Self>;
    type CorticalDataVariance = NMFoundationPowerDataCorticalVariance<Self>;
    type NeuronDataInternal = NMFoundationPowerDataNeuronInternal<Self>;
    type NeuronDataScratch = NMFoundationPowerDataNeuronScratch<Self>;
    type NeuronDataVariance = NMFoundationPowerDataNeuronVariance<Self>;

    type QuantKind = NMFoundationPowerQuantKind;
    type QuantedKind = NMFoundationPowerQuantedKind;
    const QUANT_KIND: Self::QuantKind = NMFoundationPowerQuantKind::Float64;
    const QUANTED_KIND: Self::QuantedKind = NMFoundationPowerQuantedKind::Float64(Self);
}

impl MembranePotentialQuantization for NMFoundationPowerQuantFloat64
{
    type MembranePotentialQuant = f64;
}

impl From<NMFoundationPowerQuantFloat64> for NMFoundationPowerQuantedKind {
    fn from(value: NMFoundationPowerQuantFloat64) -> Self {
        NMFoundationPowerQuantedKind::Float64(value)
    }
}

impl TryFrom<NMFoundationPowerQuantedKind> for NMFoundationPowerQuantFloat64 {
    type Error = NeuronModelEnumConversionError<NMFoundationPowerQuantKind>;

    fn try_from(value: NMFoundationPowerQuantedKind) -> Result<Self, Self::Error> {
        let actual = value.quant_kind();
        match value {
            NMFoundationPowerQuantedKind::Float64(s) => {Ok(s)},
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMFoundationPowerQuantKind::Float64,
                    actual,
                })
            }
        }
    }
}

//region Model Level Conversions

impl From<NMFoundationPowerQuantKind> for NMKind {
    fn from(value: NMFoundationPowerQuantKind) -> Self {
        NMKind::FoundationPower(value)
    }
}

impl TryFrom<NMKind> for NMFoundationPowerQuantKind {
    type Error = NeuronModelEnumConversionError<NMModelKind>;

    fn try_from(value: NMKind) -> Result<Self, Self::Error> {
        let actual = value.model_kind();
        match value {
            NMKind::FoundationPower(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMModelKind::FoundationPower,
                    actual,
                })
            }
        }
    }
}

impl From<NMFoundationPowerQuantedKind> for NMQuantedKind {
    fn from(value: NMFoundationPowerQuantedKind) -> Self {
        NMQuantedKind::FoundationPower(value)
    }
}

impl TryFrom<NMQuantedKind> for NMFoundationPowerQuantedKind {
    type Error = NeuronModelEnumConversionError<NMModelKind>;

    fn try_from(value: NMQuantedKind) -> Result<Self, Self::Error> {
        let actual = value.model_kind();
        match value {
            NMQuantedKind::FoundationPower(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMModelKind::FoundationPower,
                    actual,
                })
            }
        }
    }
}

impl<FIQ> From<NMFoundationPowerNeurogenesisGeneratorPrecursorQuanted<FIQ>> for NMNeurogenesisGeneratorPrecursorModeled<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: NMFoundationPowerNeurogenesisGeneratorPrecursorQuanted<FIQ>) -> Self {
        NMNeurogenesisGeneratorPrecursorModeled::FoundationPower(value)
    }
}

impl<FIQ> TryFrom<NMNeurogenesisGeneratorPrecursorModeled<FIQ>> for NMFoundationPowerNeurogenesisGeneratorPrecursorQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMModelKind>;

    fn try_from(value: NMNeurogenesisGeneratorPrecursorModeled<FIQ>) -> Result<Self, Self::Error> {
        let actual = value.model_kind();
        match value {
            NMNeurogenesisGeneratorPrecursorModeled::FoundationPower(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMModelKind::FoundationPower,
                    actual,
                })
            }
        }
    }
}

impl<FIQ> From<NMFoundationPowerNeurogenesisGeneratorQuanted<FIQ>> for NMNeurogenesisGeneratorModeled<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: NMFoundationPowerNeurogenesisGeneratorQuanted<FIQ>) -> Self {
        NMNeurogenesisGeneratorModeled::FoundationPower(value)
    }
}

impl<FIQ> TryFrom<NMNeurogenesisGeneratorModeled<FIQ>> for NMFoundationPowerNeurogenesisGeneratorQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMModelKind>;

    fn try_from(value: NMNeurogenesisGeneratorModeled<FIQ>) -> Result<Self, Self::Error> {
        let actual = value.model_kind();
        match value {
            NMNeurogenesisGeneratorModeled::FoundationPower(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMModelKind::FoundationPower,
                    actual,
                })
            }
        }
    }
}

impl<FIQ> From<NMFoundationPowerNeurogenesisStorageQuanted<FIQ>> for NMNeurogenesisStorageModeled<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: NMFoundationPowerNeurogenesisStorageQuanted<FIQ>) -> Self {
        NMNeurogenesisStorageModeled::FoundationPower(value)
    }
}

impl<FIQ> TryFrom<NMNeurogenesisStorageModeled<FIQ>> for NMFoundationPowerNeurogenesisStorageQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMModelKind>;

    fn try_from(value: NMNeurogenesisStorageModeled<FIQ>) -> Result<Self, Self::Error> {
        let actual = value.model_kind();
        match value {
            NMNeurogenesisStorageModeled::FoundationPower(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMModelKind::FoundationPower,
                    actual,
                })
            }
        }
    }
}

impl<FIQ> From<NMFoundationPowerNeurogenesisStorageOrGeneratorQuanted<FIQ>> for NMNeurogenesisStorageOrGeneratorModeled<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: NMFoundationPowerNeurogenesisStorageOrGeneratorQuanted<FIQ>) -> Self {
        NMNeurogenesisStorageOrGeneratorModeled::FoundationPower(value)
    }
}

impl<FIQ> TryFrom<NMNeurogenesisStorageOrGeneratorModeled<FIQ>> for NMFoundationPowerNeurogenesisStorageOrGeneratorQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMModelKind>;

    fn try_from(value: NMNeurogenesisStorageOrGeneratorModeled<FIQ>) -> Result<Self, Self::Error> {
        let actual = value.model_kind();
        match value {
            NMNeurogenesisStorageOrGeneratorModeled::FoundationPower(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMModelKind::FoundationPower,
                    actual,
                })
            }
        }
    }
}

//endregion
