use crate::neuron_models::models::foundation_power::{NMFoundationPowerDataCorticalInternal, NMFoundationPowerDataCorticalProperties, NMFoundationPowerDataCorticalScratch, NMFoundationPowerDataCorticalVariance, NMFoundationPowerDataNeuronInternal, NMFoundationPowerDataNeuronScratch, NMFoundationPowerDataNeuronVariance, NMFoundationPowerTrait};
use crate::neuron_models::models::{NMKind, NMModelKind, NMNeurogenesisGeneratorModeled, NMNeurogenesisStorageModeled, NMNeurogenesisStorageOrGeneratorModeled, NeuronModelEnumConversionError};
use crate::neuron_models::models::{NeurogenesisByGeneratorTypeOf, NeurogenesisFromStorageOf, NeurogenesisFromStorageOrByGeneratorOf};
use crate::neuron_models::neurogenesis_generator_uniform_defaults::NeurogenesisGeneratorUniformDefault;
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

    type CorticalDataProperties = NMFoundationPowerDataCorticalProperties<Self>;
    type CorticalDataInternal = NMFoundationPowerDataCorticalInternal<Self>;
    type CorticalDataScratch = NMFoundationPowerDataCorticalScratch<Self>;
    type CorticalDataVariance = NMFoundationPowerDataCorticalVariance<Self>;
    type NeuronDataInternal = NMFoundationPowerDataNeuronInternal<Self>;
    type NeuronDataScratch = NMFoundationPowerDataNeuronScratch<Self>;
    type NeuronDataVariance = NMFoundationPowerDataNeuronVariance<Self>;

    type QuantKind = NMFoundationPowerQuantKind;
    const QUANT_KIND: Self::QuantKind = NMFoundationPowerQuantKind::Float16;
}

impl MembranePotentialQuantization for NMFoundationPowerQuantFloat16
{
    type MembranePotentialQuant = f16;
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

    type CorticalDataProperties = NMFoundationPowerDataCorticalProperties<Self>;
    type CorticalDataInternal = NMFoundationPowerDataCorticalInternal<Self>;
    type CorticalDataScratch = NMFoundationPowerDataCorticalScratch<Self>;
    type CorticalDataVariance = NMFoundationPowerDataCorticalVariance<Self>;
    type NeuronDataInternal = NMFoundationPowerDataNeuronInternal<Self>;
    type NeuronDataScratch = NMFoundationPowerDataNeuronScratch<Self>;
    type NeuronDataVariance = NMFoundationPowerDataNeuronVariance<Self>;

    type QuantKind = NMFoundationPowerQuantKind;
    const QUANT_KIND: Self::QuantKind = NMFoundationPowerQuantKind::BFloat16;
}

impl MembranePotentialQuantization for NMFoundationPowerQuantBFloat16
{
    type MembranePotentialQuant = bf16;
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

    type CorticalDataProperties = NMFoundationPowerDataCorticalProperties<Self>;
    type CorticalDataInternal = NMFoundationPowerDataCorticalInternal<Self>;
    type CorticalDataScratch = NMFoundationPowerDataCorticalScratch<Self>;
    type CorticalDataVariance = NMFoundationPowerDataCorticalVariance<Self>;
    type NeuronDataInternal = NMFoundationPowerDataNeuronInternal<Self>;
    type NeuronDataScratch = NMFoundationPowerDataNeuronScratch<Self>;
    type NeuronDataVariance = NMFoundationPowerDataNeuronVariance<Self>;

    type QuantKind = NMFoundationPowerQuantKind;
    const QUANT_KIND: Self::QuantKind = NMFoundationPowerQuantKind::Float32;
}

impl MembranePotentialQuantization for NMFoundationPowerQuantFloat32
{
    type MembranePotentialQuant = f32;
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

    type CorticalDataProperties = NMFoundationPowerDataCorticalProperties<Self>;
    type CorticalDataInternal = NMFoundationPowerDataCorticalInternal<Self>;
    type CorticalDataScratch = NMFoundationPowerDataCorticalScratch<Self>;
    type CorticalDataVariance = NMFoundationPowerDataCorticalVariance<Self>;
    type NeuronDataInternal = NMFoundationPowerDataNeuronInternal<Self>;
    type NeuronDataScratch = NMFoundationPowerDataNeuronScratch<Self>;
    type NeuronDataVariance = NMFoundationPowerDataNeuronVariance<Self>;

    type QuantKind = NMFoundationPowerQuantKind;
    const QUANT_KIND: Self::QuantKind = NMFoundationPowerQuantKind::Float64;
}

impl MembranePotentialQuantization for NMFoundationPowerQuantFloat64
{
    type MembranePotentialQuant = f64;
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
