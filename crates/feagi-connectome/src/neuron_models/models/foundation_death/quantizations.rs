use crate::neuron_models::models::foundation_death::{NMFoundationDeathDataCorticalInternal, NMFoundationDeathDataCorticalProperties, NMFoundationDeathDataCorticalScratch, NMFoundationDeathDataCorticalVariance, NMFoundationDeathDataNeuronInternal, NMFoundationDeathDataNeuronScratch, NMFoundationDeathDataNeuronVariance, NMFoundationDeathTrait};
use crate::neuron_models::models::kind::{NMKind, NMModelKind};
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
    const QUANT_KIND: Self::QuantKind = NMFoundationDeathQuantKind::Float16;
}

impl MembranePotentialQuantization for NMFoundationDeathQuantFloat16
{
    type MembranePotentialQuant = f16;
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
    const QUANT_KIND: Self::QuantKind = NMFoundationDeathQuantKind::BFloat16;
}

impl MembranePotentialQuantization for NMFoundationDeathQuantBFloat16
{
    type MembranePotentialQuant = bf16;
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
    const QUANT_KIND: Self::QuantKind = NMFoundationDeathQuantKind::Float32;
}

impl MembranePotentialQuantization for NMFoundationDeathQuantFloat32
{
    type MembranePotentialQuant = f32;
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
    const QUANT_KIND: Self::QuantKind = NMFoundationDeathQuantKind::Float64;
}

impl MembranePotentialQuantization for NMFoundationDeathQuantFloat64
{
    type MembranePotentialQuant = f64;
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
