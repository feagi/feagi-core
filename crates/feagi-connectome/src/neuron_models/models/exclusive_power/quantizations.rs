use crate::neuron_models::models::exclusive_power::{NMExclusivePowerDataCorticalInternal, NMExclusivePowerDataCorticalProperties, NMExclusivePowerDataCorticalScratch, NMExclusivePowerDataCorticalVariance, NMExclusivePowerDataNeuronInternal, NMExclusivePowerDataNeuronScratch, NMExclusivePowerDataNeuronVariance, NMExclusivePowerTrait};
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


//region Generators

/// The data needed to create the generator of the Exclusive Power Neuron Model, for a given quantization
#[derive(Debug, Clone)]
pub enum NMExclusivePowerNeurogenesisGeneratorPrecursorQuanted<FIQ: FeagiIndexQuantization> {
    Float16(NeurogenesisByGeneratorPrecursorTypeOf<FIQ, NMExclusivePowerQuantFloat16>),
    BFloat16(NeurogenesisByGeneratorPrecursorTypeOf<FIQ, NMExclusivePowerQuantBFloat16>),
    Float32(NeurogenesisByGeneratorPrecursorTypeOf<FIQ, NMExclusivePowerQuantFloat32>),
    Float64(NeurogenesisByGeneratorPrecursorTypeOf<FIQ, NMExclusivePowerQuantFloat64>)
}

impl<FIQ: FeagiIndexQuantization> NMExclusivePowerNeurogenesisGeneratorPrecursorQuanted<FIQ> {
    /// The quantization of the precursor held
    pub const fn quant_kind(&self) -> NMExclusivePowerQuantKind {
        match self {
            Self::Float16(_) => NMExclusivePowerQuantKind::Float16,
            Self::BFloat16(_) => NMExclusivePowerQuantKind::BFloat16,
            Self::Float32(_) => NMExclusivePowerQuantKind::Float32,
            Self::Float64(_) => NMExclusivePowerQuantKind::Float64,
        }
    }
}

/// Allows Neurogenesis of the Exclusive Power Neuron Model by its generator for a given quantization
#[derive(Debug, Clone, Serialize, Deserialize)]

#[serde(bound = "")]
pub enum NMExclusivePowerNeurogenesisGeneratorQuanted<FIQ: FeagiIndexQuantization> {
    Float16(NeurogenesisByGeneratorTypeOf<FIQ, NMExclusivePowerQuantFloat16>),
    BFloat16(NeurogenesisByGeneratorTypeOf<FIQ, NMExclusivePowerQuantBFloat16>),
    Float32(NeurogenesisByGeneratorTypeOf<FIQ, NMExclusivePowerQuantFloat32>),
    Float64(NeurogenesisByGeneratorTypeOf<FIQ, NMExclusivePowerQuantFloat64>)
}

impl<FIQ: FeagiIndexQuantization> NMExclusivePowerNeurogenesisGeneratorQuanted<FIQ> {
    /// The quantization of the generator held
    pub const fn quant_kind(&self) -> NMExclusivePowerQuantKind {
        match self {
            Self::Float16(_) => NMExclusivePowerQuantKind::Float16,
            Self::BFloat16(_) => NMExclusivePowerQuantKind::BFloat16,
            Self::Float32(_) => NMExclusivePowerQuantKind::Float32,
            Self::Float64(_) => NMExclusivePowerQuantKind::Float64,
        }
    }
}

/// Allows Neurogenesis of the Exclusive Power Neuron Model from its stored data for a given quantization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum NMExclusivePowerNeurogenesisStorageQuanted<FIQ: FeagiIndexQuantization> {
    Float16(NeurogenesisFromStorageOf<FIQ, NMExclusivePowerQuantFloat16>),
    BFloat16(NeurogenesisFromStorageOf<FIQ, NMExclusivePowerQuantBFloat16>),
    Float32(NeurogenesisFromStorageOf<FIQ, NMExclusivePowerQuantFloat32>),
    Float64(NeurogenesisFromStorageOf<FIQ, NMExclusivePowerQuantFloat64>)
}

impl<FIQ: FeagiIndexQuantization> NMExclusivePowerNeurogenesisStorageQuanted<FIQ> {
    /// The quantization of the storage held
    pub const fn quant_kind(&self) -> NMExclusivePowerQuantKind {
        match self {
            Self::Float16(_) => NMExclusivePowerQuantKind::Float16,
            Self::BFloat16(_) => NMExclusivePowerQuantKind::BFloat16,
            Self::Float32(_) => NMExclusivePowerQuantKind::Float32,
            Self::Float64(_) => NMExclusivePowerQuantKind::Float64,
        }
    }
}

/// Allows Neurogenesis of the Exclusive Power Neuron Model from either its stored data or by its
/// generator, for a given quantization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum NMExclusivePowerNeurogenesisStorageOrGeneratorQuanted<FIQ: FeagiIndexQuantization> {
    Float16(NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMExclusivePowerQuantFloat16>),
    BFloat16(NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMExclusivePowerQuantBFloat16>),
    Float32(NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMExclusivePowerQuantFloat32>),
    Float64(NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMExclusivePowerQuantFloat64>)
}

impl<FIQ: FeagiIndexQuantization> NMExclusivePowerNeurogenesisStorageOrGeneratorQuanted<FIQ> {
    /// The quantization of the storage or generator held
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

    type NeurogenesisScratchVarianceGeneratorType<FIQ: FeagiIndexQuantization> =
        super::neurogenesis::ExclusivePowerNeurogenesisDefaults<FIQ, Self>;

    type NeurogenesisGenerator<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorUniformDefault<FIQ, Self>;
    type NeurogenesisGeneratorPrecursor<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorPrecursorUniformDefault<FIQ, Self>;

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

    type NeurogenesisScratchVarianceGeneratorType<FIQ: FeagiIndexQuantization> =
        super::neurogenesis::ExclusivePowerNeurogenesisDefaults<FIQ, Self>;

    type NeurogenesisGenerator<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorUniformDefault<FIQ, Self>;
    type NeurogenesisGeneratorPrecursor<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorPrecursorUniformDefault<FIQ, Self>;

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

    type NeurogenesisScratchVarianceGeneratorType<FIQ: FeagiIndexQuantization> =
        super::neurogenesis::ExclusivePowerNeurogenesisDefaults<FIQ, Self>;

    type NeurogenesisGenerator<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorUniformDefault<FIQ, Self>;
    type NeurogenesisGeneratorPrecursor<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorPrecursorUniformDefault<FIQ, Self>;

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

    type NeurogenesisScratchVarianceGeneratorType<FIQ: FeagiIndexQuantization> =
        super::neurogenesis::ExclusivePowerNeurogenesisDefaults<FIQ, Self>;

    type NeurogenesisGenerator<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorUniformDefault<FIQ, Self>;
    type NeurogenesisGeneratorPrecursor<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorPrecursorUniformDefault<FIQ, Self>;

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
    type Error = NeuronModelEnumConversionError<NMModelKind>;

    fn try_from(value: NMKind) -> Result<Self, Self::Error> {
        let actual = value.model_kind();
        match value {
            NMKind::ExclusivePower(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMModelKind::ExclusivePower,
                    actual,
                })
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
    type Error = NeuronModelEnumConversionError<NMModelKind>;

    fn try_from(value: NMQuantedKind) -> Result<Self, Self::Error> {
        let actual = value.model_kind();
        match value {
            NMQuantedKind::ExclusivePower(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMModelKind::ExclusivePower,
                    actual,
                })
            }
        }
    }
}

impl<FIQ> From<NMExclusivePowerNeurogenesisGeneratorPrecursorQuanted<FIQ>> for NMNeurogenesisGeneratorPrecursorModeled<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: NMExclusivePowerNeurogenesisGeneratorPrecursorQuanted<FIQ>) -> Self {
        NMNeurogenesisGeneratorPrecursorModeled::ExclusivePower(value)
    }
}

impl<FIQ> TryFrom<NMNeurogenesisGeneratorPrecursorModeled<FIQ>> for NMExclusivePowerNeurogenesisGeneratorPrecursorQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMModelKind>;

    fn try_from(value: NMNeurogenesisGeneratorPrecursorModeled<FIQ>) -> Result<Self, Self::Error> {
        let actual = value.model_kind();
        match value {
            NMNeurogenesisGeneratorPrecursorModeled::ExclusivePower(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMModelKind::ExclusivePower,
                    actual,
                })
            }
        }
    }
}

impl<FIQ> From<NMExclusivePowerNeurogenesisGeneratorQuanted<FIQ>> for NMNeurogenesisGeneratorModeled<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: NMExclusivePowerNeurogenesisGeneratorQuanted<FIQ>) -> Self {
        NMNeurogenesisGeneratorModeled::ExclusivePower(value)
    }
}

impl<FIQ> TryFrom<NMNeurogenesisGeneratorModeled<FIQ>> for NMExclusivePowerNeurogenesisGeneratorQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMModelKind>;

    fn try_from(value: NMNeurogenesisGeneratorModeled<FIQ>) -> Result<Self, Self::Error> {
        let actual = value.model_kind();
        match value {
            NMNeurogenesisGeneratorModeled::ExclusivePower(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMModelKind::ExclusivePower,
                    actual,
                })
            }
        }
    }
}

impl<FIQ> From<NMExclusivePowerNeurogenesisStorageQuanted<FIQ>> for NMNeurogenesisStorageModeled<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: NMExclusivePowerNeurogenesisStorageQuanted<FIQ>) -> Self {
        NMNeurogenesisStorageModeled::ExclusivePower(value)
    }
}

impl<FIQ> TryFrom<NMNeurogenesisStorageModeled<FIQ>> for NMExclusivePowerNeurogenesisStorageQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMModelKind>;

    fn try_from(value: NMNeurogenesisStorageModeled<FIQ>) -> Result<Self, Self::Error> {
        let actual = value.model_kind();
        match value {
            NMNeurogenesisStorageModeled::ExclusivePower(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMModelKind::ExclusivePower,
                    actual,
                })
            }
        }
    }
}

impl<FIQ> From<NMExclusivePowerNeurogenesisStorageOrGeneratorQuanted<FIQ>> for NMNeurogenesisStorageOrGeneratorModeled<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: NMExclusivePowerNeurogenesisStorageOrGeneratorQuanted<FIQ>) -> Self {
        NMNeurogenesisStorageOrGeneratorModeled::ExclusivePower(value)
    }
}

impl<FIQ> TryFrom<NMNeurogenesisStorageOrGeneratorModeled<FIQ>> for NMExclusivePowerNeurogenesisStorageOrGeneratorQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMModelKind>;

    fn try_from(value: NMNeurogenesisStorageOrGeneratorModeled<FIQ>) -> Result<Self, Self::Error> {
        let actual = value.model_kind();
        match value {
            NMNeurogenesisStorageOrGeneratorModeled::ExclusivePower(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMModelKind::ExclusivePower,
                    actual,
                })
            }
        }
    }
}

//endregion
