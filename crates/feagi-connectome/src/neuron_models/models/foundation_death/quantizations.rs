use crate::neuron_models::models::foundation_death::{NMFoundationDeathDataCorticalInternal, NMFoundationDeathDataCorticalProperties, NMFoundationDeathDataCorticalScratch, NMFoundationDeathDataCorticalVariance, NMFoundationDeathDataNeuronInternal, NMFoundationDeathDataNeuronScratch, NMFoundationDeathDataNeuronVariance, NMFoundationDeathTrait};
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
/// Describes the quantization of the Foundation Death Neuron Model
pub enum NMFoundationDeathQuantKind {
    Float16,
    BFloat16,
    Float32,
    Float64,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum NMFoundationDeathQuantedKind {
    Float16(NMFoundationDeathQuantFloat16),
    BFloat16(NMFoundationDeathQuantBFloat16),
    Float32(NMFoundationDeathQuantFloat32),
    Float64(NMFoundationDeathQuantFloat64)
}

impl NMFoundationDeathQuantedKind {
    pub fn quant_kind(&self) -> NMFoundationDeathQuantKind {
        match &self {
            NMFoundationDeathQuantedKind::Float16(_) => NMFoundationDeathQuantKind::Float16,
            NMFoundationDeathQuantedKind::BFloat16(_) => NMFoundationDeathQuantKind::BFloat16,
            NMFoundationDeathQuantedKind::Float32(_) => NMFoundationDeathQuantKind::Float32,
            NMFoundationDeathQuantedKind::Float64(_) => NMFoundationDeathQuantKind::Float64,
        }
    }

    pub fn from_nm_quant<NM: NMFoundationDeathTrait>() -> NMFoundationDeathQuantedKind {
        match NM::QUANT_KIND {
            NMFoundationDeathQuantKind::Float16 => NMFoundationDeathQuantedKind::Float16(NMFoundationDeathQuantFloat16),
            NMFoundationDeathQuantKind::BFloat16 => NMFoundationDeathQuantedKind::BFloat16(NMFoundationDeathQuantBFloat16),
            NMFoundationDeathQuantKind::Float32 => NMFoundationDeathQuantedKind::Float32(NMFoundationDeathQuantFloat32),
            NMFoundationDeathQuantKind::Float64 => NMFoundationDeathQuantedKind::Float64(NMFoundationDeathQuantFloat64),
        }
    }
}


//region Generators

/// The data needed to create the generator of the Foundation Death Neuron Model, for a given quantization
#[derive(Debug, Clone)]
pub enum NMFoundationDeathNeurogenesisGeneratorPrecursorQuanted<FIQ: FeagiIndexQuantization> {
    Float16(NeurogenesisByGeneratorPrecursorTypeOf<FIQ, NMFoundationDeathQuantFloat16>),
    BFloat16(NeurogenesisByGeneratorPrecursorTypeOf<FIQ, NMFoundationDeathQuantBFloat16>),
    Float32(NeurogenesisByGeneratorPrecursorTypeOf<FIQ, NMFoundationDeathQuantFloat32>),
    Float64(NeurogenesisByGeneratorPrecursorTypeOf<FIQ, NMFoundationDeathQuantFloat64>)
}

impl<FIQ: FeagiIndexQuantization> NMFoundationDeathNeurogenesisGeneratorPrecursorQuanted<FIQ> {
    /// The quantization of the precursor held
    pub const fn quant_kind(&self) -> NMFoundationDeathQuantKind {
        match self {
            Self::Float16(_) => NMFoundationDeathQuantKind::Float16,
            Self::BFloat16(_) => NMFoundationDeathQuantKind::BFloat16,
            Self::Float32(_) => NMFoundationDeathQuantKind::Float32,
            Self::Float64(_) => NMFoundationDeathQuantKind::Float64,
        }
    }
}

/// Allows Neurogenesis of the Foundation Death Neuron Model by its generator for a given quantization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum NMFoundationDeathNeurogenesisGeneratorQuanted<FIQ: FeagiIndexQuantization> {
    Float16(NeurogenesisByGeneratorTypeOf<FIQ, NMFoundationDeathQuantFloat16>),
    BFloat16(NeurogenesisByGeneratorTypeOf<FIQ, NMFoundationDeathQuantBFloat16>),
    Float32(NeurogenesisByGeneratorTypeOf<FIQ, NMFoundationDeathQuantFloat32>),
    Float64(NeurogenesisByGeneratorTypeOf<FIQ, NMFoundationDeathQuantFloat64>)
}

impl<FIQ: FeagiIndexQuantization> NMFoundationDeathNeurogenesisGeneratorQuanted<FIQ> {
    /// The quantization of the generator held
    pub const fn quant_kind(&self) -> NMFoundationDeathQuantKind {
        match self {
            Self::Float16(_) => NMFoundationDeathQuantKind::Float16,
            Self::BFloat16(_) => NMFoundationDeathQuantKind::BFloat16,
            Self::Float32(_) => NMFoundationDeathQuantKind::Float32,
            Self::Float64(_) => NMFoundationDeathQuantKind::Float64,
        }
    }
}

/// Allows Neurogenesis of the Foundation Death Neuron Model from its stored data for a given quantization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum NMFoundationDeathNeurogenesisStorageQuanted<FIQ: FeagiIndexQuantization> {
    Float16(NeurogenesisFromStorageOf<FIQ, NMFoundationDeathQuantFloat16>),
    BFloat16(NeurogenesisFromStorageOf<FIQ, NMFoundationDeathQuantBFloat16>),
    Float32(NeurogenesisFromStorageOf<FIQ, NMFoundationDeathQuantFloat32>),
    Float64(NeurogenesisFromStorageOf<FIQ, NMFoundationDeathQuantFloat64>)
}

impl<FIQ: FeagiIndexQuantization> NMFoundationDeathNeurogenesisStorageQuanted<FIQ> {
    /// The quantization of the storage held
    pub const fn quant_kind(&self) -> NMFoundationDeathQuantKind {
        match self {
            Self::Float16(_) => NMFoundationDeathQuantKind::Float16,
            Self::BFloat16(_) => NMFoundationDeathQuantKind::BFloat16,
            Self::Float32(_) => NMFoundationDeathQuantKind::Float32,
            Self::Float64(_) => NMFoundationDeathQuantKind::Float64,
        }
    }
}

/// Allows Neurogenesis of the Foundation Death Neuron Model from either its stored data or by its
/// generator, for a given quantization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum NMFoundationDeathNeurogenesisStorageOrGeneratorQuanted<FIQ: FeagiIndexQuantization> {
    Float16(NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMFoundationDeathQuantFloat16>),
    BFloat16(NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMFoundationDeathQuantBFloat16>),
    Float32(NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMFoundationDeathQuantFloat32>),
    Float64(NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMFoundationDeathQuantFloat64>)
}

impl<FIQ: FeagiIndexQuantization> NMFoundationDeathNeurogenesisStorageOrGeneratorQuanted<FIQ> {
    /// The quantization of the storage or generator held
    pub const fn quant_kind(&self) -> NMFoundationDeathQuantKind {
        match self {
            Self::Float16(_) => NMFoundationDeathQuantKind::Float16,
            Self::BFloat16(_) => NMFoundationDeathQuantKind::BFloat16,
            Self::Float32(_) => NMFoundationDeathQuantKind::Float32,
            Self::Float64(_) => NMFoundationDeathQuantKind::Float64,
        }
    }
}


//endregion

//endregion

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMFoundationDeathQuantFloat16;

impl NMFoundationDeathTrait for NMFoundationDeathQuantFloat16 {
}

impl NeuronModel for NMFoundationDeathQuantFloat16 {
    type NeuronLayoutTag = NeuronLayoutTagVoxel;

    type NeurogenesisScratchVarianceGeneratorType<FIQ: FeagiIndexQuantization> =
        super::neurogenesis::FoundationDeathNeurogenesisDefaults<FIQ, Self>;

    type NeurogenesisGenerator<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorUniformDefault<FIQ, Self>;
    type NeurogenesisGeneratorPrecursor<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorPrecursorUniformDefault<FIQ, Self>;

    type CorticalDataProperties = NMFoundationDeathDataCorticalProperties<Self>;
    type CorticalDataInternal = NMFoundationDeathDataCorticalInternal<Self>;
    type CorticalDataScratch = NMFoundationDeathDataCorticalScratch<Self>;
    type CorticalDataVariance = NMFoundationDeathDataCorticalVariance<Self>;
    type NeuronDataInternal = NMFoundationDeathDataNeuronInternal<Self>;
    type NeuronDataScratch = NMFoundationDeathDataNeuronScratch<Self>;
    type NeuronDataVariance = NMFoundationDeathDataNeuronVariance<Self>;

    type QuantKind = NMFoundationDeathQuantKind;
    type QuantedKind = NMFoundationDeathQuantedKind;
    const QUANT_KIND: Self::QuantKind = NMFoundationDeathQuantKind::Float16;
    const QUANTED_KIND: Self::QuantedKind = NMFoundationDeathQuantedKind::Float16(Self);
}

impl MembranePotentialQuantization for NMFoundationDeathQuantFloat16
{
    type MembranePotentialQuant = f16;
}

impl From<NMFoundationDeathQuantFloat16> for NMFoundationDeathQuantedKind {
    fn from(value: NMFoundationDeathQuantFloat16) -> Self {
        NMFoundationDeathQuantedKind::Float16(value)
    }
}

impl TryFrom<NMFoundationDeathQuantedKind> for NMFoundationDeathQuantFloat16 {
    type Error = NeuronModelEnumConversionError<NMFoundationDeathQuantKind>;

    fn try_from(value: NMFoundationDeathQuantedKind) -> Result<Self, Self::Error> {
        let actual = value.quant_kind();
        match value {
            NMFoundationDeathQuantedKind::Float16(s) => {Ok(s)},
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMFoundationDeathQuantKind::Float16,
                    actual,
                })
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMFoundationDeathQuantBFloat16;

impl NMFoundationDeathTrait for NMFoundationDeathQuantBFloat16 {
}

impl NeuronModel for NMFoundationDeathQuantBFloat16 {
    type NeuronLayoutTag = NeuronLayoutTagVoxel;

    type NeurogenesisScratchVarianceGeneratorType<FIQ: FeagiIndexQuantization> =
        super::neurogenesis::FoundationDeathNeurogenesisDefaults<FIQ, Self>;

    type NeurogenesisGenerator<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorUniformDefault<FIQ, Self>;
    type NeurogenesisGeneratorPrecursor<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorPrecursorUniformDefault<FIQ, Self>;

    type CorticalDataProperties = NMFoundationDeathDataCorticalProperties<Self>;
    type CorticalDataInternal = NMFoundationDeathDataCorticalInternal<Self>;
    type CorticalDataScratch = NMFoundationDeathDataCorticalScratch<Self>;
    type CorticalDataVariance = NMFoundationDeathDataCorticalVariance<Self>;
    type NeuronDataInternal = NMFoundationDeathDataNeuronInternal<Self>;
    type NeuronDataScratch = NMFoundationDeathDataNeuronScratch<Self>;
    type NeuronDataVariance = NMFoundationDeathDataNeuronVariance<Self>;

    type QuantKind = NMFoundationDeathQuantKind;
    type QuantedKind = NMFoundationDeathQuantedKind;
    const QUANT_KIND: Self::QuantKind = NMFoundationDeathQuantKind::BFloat16;
    const QUANTED_KIND: Self::QuantedKind = NMFoundationDeathQuantedKind::BFloat16(Self);
}

impl MembranePotentialQuantization for NMFoundationDeathQuantBFloat16
{
    type MembranePotentialQuant = bf16;
}

impl From<NMFoundationDeathQuantBFloat16> for NMFoundationDeathQuantedKind {
    fn from(value: NMFoundationDeathQuantBFloat16) -> Self {
        NMFoundationDeathQuantedKind::BFloat16(value)
    }
}

impl TryFrom<NMFoundationDeathQuantedKind> for NMFoundationDeathQuantBFloat16 {
    type Error = NeuronModelEnumConversionError<NMFoundationDeathQuantKind>;

    fn try_from(value: NMFoundationDeathQuantedKind) -> Result<Self, Self::Error> {
        let actual = value.quant_kind();
        match value {
            NMFoundationDeathQuantedKind::BFloat16(s) => {Ok(s)},
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMFoundationDeathQuantKind::BFloat16,
                    actual,
                })
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMFoundationDeathQuantFloat32;

impl NMFoundationDeathTrait for NMFoundationDeathQuantFloat32 {
}

impl NeuronModel for NMFoundationDeathQuantFloat32 {
    type NeuronLayoutTag = NeuronLayoutTagVoxel;

    type NeurogenesisScratchVarianceGeneratorType<FIQ: FeagiIndexQuantization> =
        super::neurogenesis::FoundationDeathNeurogenesisDefaults<FIQ, Self>;

    type NeurogenesisGenerator<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorUniformDefault<FIQ, Self>;
    type NeurogenesisGeneratorPrecursor<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorPrecursorUniformDefault<FIQ, Self>;

    type CorticalDataProperties = NMFoundationDeathDataCorticalProperties<Self>;
    type CorticalDataInternal = NMFoundationDeathDataCorticalInternal<Self>;
    type CorticalDataScratch = NMFoundationDeathDataCorticalScratch<Self>;
    type CorticalDataVariance = NMFoundationDeathDataCorticalVariance<Self>;
    type NeuronDataInternal = NMFoundationDeathDataNeuronInternal<Self>;
    type NeuronDataScratch = NMFoundationDeathDataNeuronScratch<Self>;
    type NeuronDataVariance = NMFoundationDeathDataNeuronVariance<Self>;

    type QuantKind = NMFoundationDeathQuantKind;
    type QuantedKind = NMFoundationDeathQuantedKind;
    const QUANT_KIND: Self::QuantKind = NMFoundationDeathQuantKind::Float32;
    const QUANTED_KIND: Self::QuantedKind = NMFoundationDeathQuantedKind::Float32(Self);
}

impl MembranePotentialQuantization for NMFoundationDeathQuantFloat32
{
    type MembranePotentialQuant = f32;
}

impl From<NMFoundationDeathQuantFloat32> for NMFoundationDeathQuantedKind {
    fn from(value: NMFoundationDeathQuantFloat32) -> Self {
        NMFoundationDeathQuantedKind::Float32(value)
    }
}

impl TryFrom<NMFoundationDeathQuantedKind> for NMFoundationDeathQuantFloat32 {
    type Error = NeuronModelEnumConversionError<NMFoundationDeathQuantKind>;

    fn try_from(value: NMFoundationDeathQuantedKind) -> Result<Self, Self::Error> {
        let actual = value.quant_kind();
        match value {
            NMFoundationDeathQuantedKind::Float32(s) => {Ok(s)},
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMFoundationDeathQuantKind::Float32,
                    actual,
                })
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMFoundationDeathQuantFloat64;

impl NMFoundationDeathTrait for NMFoundationDeathQuantFloat64 {
}

impl NeuronModel for NMFoundationDeathQuantFloat64 {
    type NeuronLayoutTag = NeuronLayoutTagVoxel;

    type NeurogenesisScratchVarianceGeneratorType<FIQ: FeagiIndexQuantization> =
        super::neurogenesis::FoundationDeathNeurogenesisDefaults<FIQ, Self>;

    type NeurogenesisGenerator<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorUniformDefault<FIQ, Self>;
    type NeurogenesisGeneratorPrecursor<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorPrecursorUniformDefault<FIQ, Self>;

    type CorticalDataProperties = NMFoundationDeathDataCorticalProperties<Self>;
    type CorticalDataInternal = NMFoundationDeathDataCorticalInternal<Self>;
    type CorticalDataScratch = NMFoundationDeathDataCorticalScratch<Self>;
    type CorticalDataVariance = NMFoundationDeathDataCorticalVariance<Self>;
    type NeuronDataInternal = NMFoundationDeathDataNeuronInternal<Self>;
    type NeuronDataScratch = NMFoundationDeathDataNeuronScratch<Self>;
    type NeuronDataVariance = NMFoundationDeathDataNeuronVariance<Self>;

    type QuantKind = NMFoundationDeathQuantKind;
    type QuantedKind = NMFoundationDeathQuantedKind;
    const QUANT_KIND: Self::QuantKind = NMFoundationDeathQuantKind::Float64;
    const QUANTED_KIND: Self::QuantedKind = NMFoundationDeathQuantedKind::Float64(Self);
}

impl MembranePotentialQuantization for NMFoundationDeathQuantFloat64
{
    type MembranePotentialQuant = f64;
}

impl From<NMFoundationDeathQuantFloat64> for NMFoundationDeathQuantedKind {
    fn from(value: NMFoundationDeathQuantFloat64) -> Self {
        NMFoundationDeathQuantedKind::Float64(value)
    }
}

impl TryFrom<NMFoundationDeathQuantedKind> for NMFoundationDeathQuantFloat64 {
    type Error = NeuronModelEnumConversionError<NMFoundationDeathQuantKind>;

    fn try_from(value: NMFoundationDeathQuantedKind) -> Result<Self, Self::Error> {
        let actual = value.quant_kind();
        match value {
            NMFoundationDeathQuantedKind::Float64(s) => {Ok(s)},
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMFoundationDeathQuantKind::Float64,
                    actual,
                })
            }
        }
    }
}

//region Model Level Conversions

impl From<NMFoundationDeathQuantKind> for NMKind {
    fn from(value: NMFoundationDeathQuantKind) -> Self {
        NMKind::FoundationDeath(value)
    }
}

impl TryFrom<NMKind> for NMFoundationDeathQuantKind {
    type Error = NeuronModelEnumConversionError<NMModelKind>;

    fn try_from(value: NMKind) -> Result<Self, Self::Error> {
        let actual = value.model_kind();
        match value {
            NMKind::FoundationDeath(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMModelKind::FoundationDeath,
                    actual,
                })
            }
        }
    }
}

impl From<NMFoundationDeathQuantedKind> for NMQuantedKind {
    fn from(value: NMFoundationDeathQuantedKind) -> Self {
        NMQuantedKind::FoundationDeath(value)
    }
}

impl TryFrom<NMQuantedKind> for NMFoundationDeathQuantedKind {
    type Error = NeuronModelEnumConversionError<NMModelKind>;

    fn try_from(value: NMQuantedKind) -> Result<Self, Self::Error> {
        let actual = value.model_kind();
        match value {
            NMQuantedKind::FoundationDeath(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMModelKind::FoundationDeath,
                    actual,
                })
            }
        }
    }
}

impl<FIQ> From<NMFoundationDeathNeurogenesisGeneratorPrecursorQuanted<FIQ>> for NMNeurogenesisGeneratorPrecursorModeled<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: NMFoundationDeathNeurogenesisGeneratorPrecursorQuanted<FIQ>) -> Self {
        NMNeurogenesisGeneratorPrecursorModeled::FoundationDeath(value)
    }
}

impl<FIQ> TryFrom<NMNeurogenesisGeneratorPrecursorModeled<FIQ>> for NMFoundationDeathNeurogenesisGeneratorPrecursorQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMModelKind>;

    fn try_from(value: NMNeurogenesisGeneratorPrecursorModeled<FIQ>) -> Result<Self, Self::Error> {
        let actual = value.model_kind();
        match value {
            NMNeurogenesisGeneratorPrecursorModeled::FoundationDeath(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMModelKind::FoundationDeath,
                    actual,
                })
            }
        }
    }
}

impl<FIQ> From<NMFoundationDeathNeurogenesisGeneratorQuanted<FIQ>> for NMNeurogenesisGeneratorModeled<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: NMFoundationDeathNeurogenesisGeneratorQuanted<FIQ>) -> Self {
        NMNeurogenesisGeneratorModeled::FoundationDeath(value)
    }
}

impl<FIQ> TryFrom<NMNeurogenesisGeneratorModeled<FIQ>> for NMFoundationDeathNeurogenesisGeneratorQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMModelKind>;

    fn try_from(value: NMNeurogenesisGeneratorModeled<FIQ>) -> Result<Self, Self::Error> {
        let actual = value.model_kind();
        match value {
            NMNeurogenesisGeneratorModeled::FoundationDeath(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMModelKind::FoundationDeath,
                    actual,
                })
            }
        }
    }
}

impl<FIQ> From<NMFoundationDeathNeurogenesisStorageQuanted<FIQ>> for NMNeurogenesisStorageModeled<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: NMFoundationDeathNeurogenesisStorageQuanted<FIQ>) -> Self {
        NMNeurogenesisStorageModeled::FoundationDeath(value)
    }
}

impl<FIQ> TryFrom<NMNeurogenesisStorageModeled<FIQ>> for NMFoundationDeathNeurogenesisStorageQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMModelKind>;

    fn try_from(value: NMNeurogenesisStorageModeled<FIQ>) -> Result<Self, Self::Error> {
        let actual = value.model_kind();
        match value {
            NMNeurogenesisStorageModeled::FoundationDeath(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMModelKind::FoundationDeath,
                    actual,
                })
            }
        }
    }
}

impl<FIQ> From<NMFoundationDeathNeurogenesisStorageOrGeneratorQuanted<FIQ>> for NMNeurogenesisStorageOrGeneratorModeled<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: NMFoundationDeathNeurogenesisStorageOrGeneratorQuanted<FIQ>) -> Self {
        NMNeurogenesisStorageOrGeneratorModeled::FoundationDeath(value)
    }
}

impl<FIQ> TryFrom<NMNeurogenesisStorageOrGeneratorModeled<FIQ>> for NMFoundationDeathNeurogenesisStorageOrGeneratorQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMModelKind>;

    fn try_from(value: NMNeurogenesisStorageOrGeneratorModeled<FIQ>) -> Result<Self, Self::Error> {
        let actual = value.model_kind();
        match value {
            NMNeurogenesisStorageOrGeneratorModeled::FoundationDeath(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMModelKind::FoundationDeath,
                    actual,
                })
            }
        }
    }
}

//endregion
