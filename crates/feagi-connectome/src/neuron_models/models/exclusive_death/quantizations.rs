use crate::neuron_models::models::exclusive_death::{NMExclusiveDeathDataCorticalInternal, NMExclusiveDeathDataCorticalProperties, NMExclusiveDeathDataCorticalScratch, NMExclusiveDeathDataCorticalVariance, NMExclusiveDeathDataNeuronInternal, NMExclusiveDeathDataNeuronScratch, NMExclusiveDeathDataNeuronVariance, NMExclusiveDeathTrait};
use crate::neuron_models::models::kind::{NMKind, NMQuantedKind};
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


//region Generators

/// The data needed to create the generator of the Exclusive Death Neuron Model, for a given quantization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum NMExclusiveDeathQuantedNeurogenesisGeneratorPrecursor<FIQ: FeagiIndexQuantization> {
    Float16(NeurogenesisByGeneratorPrecursorTypeOf<FIQ, NMExclusiveDeathQuantFloat16>),
    BFloat16(NeurogenesisByGeneratorPrecursorTypeOf<FIQ, NMExclusiveDeathQuantBFloat16>),
    Float32(NeurogenesisByGeneratorPrecursorTypeOf<FIQ, NMExclusiveDeathQuantFloat32>),
    Float64(NeurogenesisByGeneratorPrecursorTypeOf<FIQ, NMExclusiveDeathQuantFloat64>)
}

impl<FIQ: FeagiIndexQuantization> NMExclusiveDeathQuantedNeurogenesisGeneratorPrecursor<FIQ> {
    /// The quantization of the precursor held
    pub const fn quant_kind(&self) -> NMExclusiveDeathQuantKind {
        match self {
            Self::Float16(_) => NMExclusiveDeathQuantKind::Float16,
            Self::BFloat16(_) => NMExclusiveDeathQuantKind::BFloat16,
            Self::Float32(_) => NMExclusiveDeathQuantKind::Float32,
            Self::Float64(_) => NMExclusiveDeathQuantKind::Float64,
        }
    }
}

/// Allows Neurogenesis of the Exclusive Death Neuron Model by its generator for a given quantization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum NMExclusiveDeathNeurogenesisGeneratorQuanted<FIQ: FeagiIndexQuantization> {
    Float16(NeurogenesisByGeneratorTypeOf<FIQ, NMExclusiveDeathQuantFloat16>),
    BFloat16(NeurogenesisByGeneratorTypeOf<FIQ, NMExclusiveDeathQuantBFloat16>),
    Float32(NeurogenesisByGeneratorTypeOf<FIQ, NMExclusiveDeathQuantFloat32>),
    Float64(NeurogenesisByGeneratorTypeOf<FIQ, NMExclusiveDeathQuantFloat64>)
}

impl<FIQ: FeagiIndexQuantization> NMExclusiveDeathNeurogenesisGeneratorQuanted<FIQ> {
    /// The quantization of the generator held
    pub const fn quant_kind(&self) -> NMExclusiveDeathQuantKind {
        match self {
            Self::Float16(_) => NMExclusiveDeathQuantKind::Float16,
            Self::BFloat16(_) => NMExclusiveDeathQuantKind::BFloat16,
            Self::Float32(_) => NMExclusiveDeathQuantKind::Float32,
            Self::Float64(_) => NMExclusiveDeathQuantKind::Float64,
        }
    }
}

/// Allows Neurogenesis of the Exclusive Death Neuron Model from its stored data for a given quantization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum NMExclusiveDeathNeurogenesisStorageQuanted<FIQ: FeagiIndexQuantization> {
    Float16(NeurogenesisFromStorageOf<FIQ, NMExclusiveDeathQuantFloat16>),
    BFloat16(NeurogenesisFromStorageOf<FIQ, NMExclusiveDeathQuantBFloat16>),
    Float32(NeurogenesisFromStorageOf<FIQ, NMExclusiveDeathQuantFloat32>),
    Float64(NeurogenesisFromStorageOf<FIQ, NMExclusiveDeathQuantFloat64>)
}

impl<FIQ: FeagiIndexQuantization> NMExclusiveDeathNeurogenesisStorageQuanted<FIQ> {
    /// The quantization of the storage held
    pub const fn quant_kind(&self) -> NMExclusiveDeathQuantKind {
        match self {
            Self::Float16(_) => NMExclusiveDeathQuantKind::Float16,
            Self::BFloat16(_) => NMExclusiveDeathQuantKind::BFloat16,
            Self::Float32(_) => NMExclusiveDeathQuantKind::Float32,
            Self::Float64(_) => NMExclusiveDeathQuantKind::Float64,
        }
    }
}

/// Allows Neurogenesis of the Exclusive Death Neuron Model from either its stored data or by its
/// generator, for a given quantization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum NMExclusiveDeathNeurogenesisStorageOrGeneratorQuanted<FIQ: FeagiIndexQuantization> {
    Float16(NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMExclusiveDeathQuantFloat16>),
    BFloat16(NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMExclusiveDeathQuantBFloat16>),
    Float32(NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMExclusiveDeathQuantFloat32>),
    Float64(NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMExclusiveDeathQuantFloat64>)
}

impl<FIQ: FeagiIndexQuantization> NMExclusiveDeathNeurogenesisStorageOrGeneratorQuanted<FIQ> {
    /// The quantization of the storage or generator held
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

    type NeurogenesisScratchVarianceGeneratorType<FIQ: FeagiIndexQuantization> =
        super::neurogenesis::ExclusiveDeathNeurogenesisDefaults<FIQ, Self>;

    type NeurogenesisGenerator<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorUniformDefault<FIQ, Self>;
    type NeurogenesisGeneratorPrecursor<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorPrecursorUniformDefault<FIQ, Self>;

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

    type NeurogenesisScratchVarianceGeneratorType<FIQ: FeagiIndexQuantization> =
        super::neurogenesis::ExclusiveDeathNeurogenesisDefaults<FIQ, Self>;

    type NeurogenesisGenerator<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorUniformDefault<FIQ, Self>;
    type NeurogenesisGeneratorPrecursor<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorPrecursorUniformDefault<FIQ, Self>;

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

    type NeurogenesisScratchVarianceGeneratorType<FIQ: FeagiIndexQuantization> =
        super::neurogenesis::ExclusiveDeathNeurogenesisDefaults<FIQ, Self>;

    type NeurogenesisGenerator<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorUniformDefault<FIQ, Self>;
    type NeurogenesisGeneratorPrecursor<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorPrecursorUniformDefault<FIQ, Self>;

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

    type NeurogenesisScratchVarianceGeneratorType<FIQ: FeagiIndexQuantization> =
        super::neurogenesis::ExclusiveDeathNeurogenesisDefaults<FIQ, Self>;

    type NeurogenesisGenerator<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorUniformDefault<FIQ, Self>;
    type NeurogenesisGeneratorPrecursor<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorPrecursorUniformDefault<FIQ, Self>;

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

impl<FIQ> From<NMExclusiveDeathQuantedNeurogenesisGeneratorPrecursor<FIQ>> for NMNeurogenesisGeneratorPrecursorModeled<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: NMExclusiveDeathQuantedNeurogenesisGeneratorPrecursor<FIQ>) -> Self {
        NMNeurogenesisGeneratorPrecursorModeled::ExclusiveDeath(value)
    }
}

impl<FIQ> TryFrom<NMNeurogenesisGeneratorPrecursorModeled<FIQ>> for NMExclusiveDeathQuantedNeurogenesisGeneratorPrecursor<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMKind>;

    fn try_from(value: NMNeurogenesisGeneratorPrecursorModeled<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMNeurogenesisGeneratorPrecursorModeled::ExclusiveDeath(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleModel)
            }
        }
    }
}

impl<FIQ> From<NMExclusiveDeathNeurogenesisGeneratorQuanted<FIQ>> for NMNeurogenesisGeneratorModeled<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: NMExclusiveDeathNeurogenesisGeneratorQuanted<FIQ>) -> Self {
        NMNeurogenesisGeneratorModeled::ExclusiveDeath(value)
    }
}

impl<FIQ> TryFrom<NMNeurogenesisGeneratorModeled<FIQ>> for NMExclusiveDeathNeurogenesisGeneratorQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMKind>;

    fn try_from(value: NMNeurogenesisGeneratorModeled<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMNeurogenesisGeneratorModeled::ExclusiveDeath(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleModel)
            }
        }
    }
}

impl<FIQ> From<NMExclusiveDeathNeurogenesisStorageQuanted<FIQ>> for NMNeurogenesisStorageModeled<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: NMExclusiveDeathNeurogenesisStorageQuanted<FIQ>) -> Self {
        NMNeurogenesisStorageModeled::ExclusiveDeath(value)
    }
}

impl<FIQ> TryFrom<NMNeurogenesisStorageModeled<FIQ>> for NMExclusiveDeathNeurogenesisStorageQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMKind>;

    fn try_from(value: NMNeurogenesisStorageModeled<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMNeurogenesisStorageModeled::ExclusiveDeath(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleModel)
            }
        }
    }
}

impl<FIQ> From<NMExclusiveDeathNeurogenesisStorageOrGeneratorQuanted<FIQ>> for NMNeurogenesisStorageOrGeneratorModeled<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: NMExclusiveDeathNeurogenesisStorageOrGeneratorQuanted<FIQ>) -> Self {
        NMNeurogenesisStorageOrGeneratorModeled::ExclusiveDeath(value)
    }
}

impl<FIQ> TryFrom<NMNeurogenesisStorageOrGeneratorModeled<FIQ>> for NMExclusiveDeathNeurogenesisStorageOrGeneratorQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMKind>;

    fn try_from(value: NMNeurogenesisStorageOrGeneratorModeled<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMNeurogenesisStorageOrGeneratorModeled::ExclusiveDeath(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleModel)
            }
        }
    }
}

//endregion
