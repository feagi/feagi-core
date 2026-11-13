use crate::neuron_models::models::exclusive_power::{
    NMExclusivePowerDataCorticalInternal, NMExclusivePowerDataCorticalProperties, NMExclusivePowerDataCorticalScratch,
    NMExclusivePowerDataCorticalVariance, NMExclusivePowerDataNeuronInternal, NMExclusivePowerDataNeuronScratch, NMExclusivePowerDataNeuronVariance,
    NMExclusivePowerTrait,
};
use crate::neuron_models::models::kind::NMKind;
use crate::neuron_models::models::{NMContainer, NMMember, NMModeled, NeuronModelEnumConversionError};
use crate::neuron_models::neurogenesis_generator_uniform_defaults::NeurogenesisGeneratorUniformDefault;
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::voxel::NeuronLayoutTagVoxel;
use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome_neural::neural_basis::quantization_levels::{FeagiIndexQuantization, MembranePotentialQuantization};
use half::{bf16, f16};

//region enums

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, ::serde::Serialize, ::serde::Deserialize)]
/// Describes the quantization of the Exclusive Power Neuron Model
pub enum NMExclusivePowerQuantKind {
    Float16,
    BFloat16,
    Float32,
    Float64,
}

//region Quanted

/// Data of container `C` for a given quantization of the Exclusive Power Neuron Model
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(bound(
    serialize = "C::Of<NMExclusivePowerQuantFloat16>: ::serde::Serialize, C::Of<NMExclusivePowerQuantBFloat16>: ::serde::Serialize, C::Of<NMExclusivePowerQuantFloat32>: ::serde::Serialize, C::Of<NMExclusivePowerQuantFloat64>: ::serde::Serialize",
    deserialize = "C::Of<NMExclusivePowerQuantFloat16>: ::serde::Deserialize<'de>, C::Of<NMExclusivePowerQuantBFloat16>: ::serde::Deserialize<'de>, C::Of<NMExclusivePowerQuantFloat32>: ::serde::Deserialize<'de>, C::Of<NMExclusivePowerQuantFloat64>: ::serde::Deserialize<'de>"
))]
pub enum NMExclusivePowerQuanted<C: NMContainer> {
    Float16(C::Of<NMExclusivePowerQuantFloat16>),
    BFloat16(C::Of<NMExclusivePowerQuantBFloat16>),
    Float32(C::Of<NMExclusivePowerQuantFloat32>),
    Float64(C::Of<NMExclusivePowerQuantFloat64>),
}

impl<C: NMContainer> NMExclusivePowerQuanted<C> {
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

impl<C: NMContainer> Clone for NMExclusivePowerQuanted<C>
where
    C::Of<NMExclusivePowerQuantFloat16>: Clone,
    C::Of<NMExclusivePowerQuantBFloat16>: Clone,
    C::Of<NMExclusivePowerQuantFloat32>: Clone,
    C::Of<NMExclusivePowerQuantFloat64>: Clone,
{
    fn clone(&self) -> Self {
        match self {
            Self::Float16(v) => Self::Float16(v.clone()),
            Self::BFloat16(v) => Self::BFloat16(v.clone()),
            Self::Float32(v) => Self::Float32(v.clone()),
            Self::Float64(v) => Self::Float64(v.clone()),
        }
    }
}

impl<C: NMContainer> core::fmt::Debug for NMExclusivePowerQuanted<C>
where
    C::Of<NMExclusivePowerQuantFloat16>: core::fmt::Debug,
    C::Of<NMExclusivePowerQuantBFloat16>: core::fmt::Debug,
    C::Of<NMExclusivePowerQuantFloat32>: core::fmt::Debug,
    C::Of<NMExclusivePowerQuantFloat64>: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Float16(v) => f.debug_tuple("Float16").field(v).finish(),
            Self::BFloat16(v) => f.debug_tuple("BFloat16").field(v).finish(),
            Self::Float32(v) => f.debug_tuple("Float32").field(v).finish(),
            Self::Float64(v) => f.debug_tuple("Float64").field(v).finish(),
        }
    }
}

impl NMMember for NMExclusivePowerQuantFloat16 {
    type Quanted<C: NMContainer> = NMExclusivePowerQuanted<C>;

    fn wrap<C: NMContainer>(data: C::Of<Self>) -> NMExclusivePowerQuanted<C> {
        NMExclusivePowerQuanted::Float16(data)
    }

    fn unwrap<C: NMContainer>(quanted: NMExclusivePowerQuanted<C>) -> Result<C::Of<Self>, NeuronModelEnumConversionError<NMExclusivePowerQuantKind>> {
        match quanted {
            NMExclusivePowerQuanted::Float16(v) => Ok(v),
            other => Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                expected: Self::QUANT_KIND,
                actual: other.quant_kind(),
            }),
        }
    }
}

impl NMMember for NMExclusivePowerQuantBFloat16 {
    type Quanted<C: NMContainer> = NMExclusivePowerQuanted<C>;

    fn wrap<C: NMContainer>(data: C::Of<Self>) -> NMExclusivePowerQuanted<C> {
        NMExclusivePowerQuanted::BFloat16(data)
    }

    fn unwrap<C: NMContainer>(quanted: NMExclusivePowerQuanted<C>) -> Result<C::Of<Self>, NeuronModelEnumConversionError<NMExclusivePowerQuantKind>> {
        match quanted {
            NMExclusivePowerQuanted::BFloat16(v) => Ok(v),
            other => Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                expected: Self::QUANT_KIND,
                actual: other.quant_kind(),
            }),
        }
    }
}

impl NMMember for NMExclusivePowerQuantFloat32 {
    type Quanted<C: NMContainer> = NMExclusivePowerQuanted<C>;

    fn wrap<C: NMContainer>(data: C::Of<Self>) -> NMExclusivePowerQuanted<C> {
        NMExclusivePowerQuanted::Float32(data)
    }

    fn unwrap<C: NMContainer>(quanted: NMExclusivePowerQuanted<C>) -> Result<C::Of<Self>, NeuronModelEnumConversionError<NMExclusivePowerQuantKind>> {
        match quanted {
            NMExclusivePowerQuanted::Float32(v) => Ok(v),
            other => Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                expected: Self::QUANT_KIND,
                actual: other.quant_kind(),
            }),
        }
    }
}

impl NMMember for NMExclusivePowerQuantFloat64 {
    type Quanted<C: NMContainer> = NMExclusivePowerQuanted<C>;

    fn wrap<C: NMContainer>(data: C::Of<Self>) -> NMExclusivePowerQuanted<C> {
        NMExclusivePowerQuanted::Float64(data)
    }

    fn unwrap<C: NMContainer>(quanted: NMExclusivePowerQuanted<C>) -> Result<C::Of<Self>, NeuronModelEnumConversionError<NMExclusivePowerQuantKind>> {
        match quanted {
            NMExclusivePowerQuanted::Float64(v) => Ok(v),
            other => Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                expected: Self::QUANT_KIND,
                actual: other.quant_kind(),
            }),
        }
    }
}

//endregion

//endregion

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMExclusivePowerQuantFloat16;

impl NMExclusivePowerTrait for NMExclusivePowerQuantFloat16 {}

impl NeuronModel for NMExclusivePowerQuantFloat16 {
    type NeuronLayoutTag = NeuronLayoutTagVoxel;

    type NeurogenesisScratchVarianceDefaults<FIQ: FeagiIndexQuantization> = super::neurogenesis::ExclusivePowerNeurogenesisDefaults<FIQ, Self>;

    type NeurogenesisByGenerator<FIQ: FeagiIndexQuantization> = NeurogenesisGeneratorUniformDefault<FIQ, Self>;

    type CorticalDataProperties = NMExclusivePowerDataCorticalProperties<Self>;
    type CorticalDataInternal = NMExclusivePowerDataCorticalInternal<Self>;
    type CorticalDataScratch = NMExclusivePowerDataCorticalScratch<Self>;
    type CorticalDataVariance = NMExclusivePowerDataCorticalVariance<Self>;
    type NeuronDataInternal = NMExclusivePowerDataNeuronInternal<Self>;
    type NeuronDataScratch = NMExclusivePowerDataNeuronScratch<Self>;
    type NeuronDataVariance = NMExclusivePowerDataNeuronVariance<Self>;

    type QuantKind = NMExclusivePowerQuantKind;
    const QUANT_KIND: Self::QuantKind = NMExclusivePowerQuantKind::Float16;
}

impl MembranePotentialQuantization for NMExclusivePowerQuantFloat16 {
    type MembranePotentialQuant = f16;
}

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMExclusivePowerQuantBFloat16;

impl NMExclusivePowerTrait for NMExclusivePowerQuantBFloat16 {}

impl NeuronModel for NMExclusivePowerQuantBFloat16 {
    type NeuronLayoutTag = NeuronLayoutTagVoxel;

    type NeurogenesisScratchVarianceDefaults<FIQ: FeagiIndexQuantization> = super::neurogenesis::ExclusivePowerNeurogenesisDefaults<FIQ, Self>;

    type NeurogenesisByGenerator<FIQ: FeagiIndexQuantization> = NeurogenesisGeneratorUniformDefault<FIQ, Self>;

    type CorticalDataProperties = NMExclusivePowerDataCorticalProperties<Self>;
    type CorticalDataInternal = NMExclusivePowerDataCorticalInternal<Self>;
    type CorticalDataScratch = NMExclusivePowerDataCorticalScratch<Self>;
    type CorticalDataVariance = NMExclusivePowerDataCorticalVariance<Self>;
    type NeuronDataInternal = NMExclusivePowerDataNeuronInternal<Self>;
    type NeuronDataScratch = NMExclusivePowerDataNeuronScratch<Self>;
    type NeuronDataVariance = NMExclusivePowerDataNeuronVariance<Self>;

    type QuantKind = NMExclusivePowerQuantKind;
    const QUANT_KIND: Self::QuantKind = NMExclusivePowerQuantKind::BFloat16;
}

impl MembranePotentialQuantization for NMExclusivePowerQuantBFloat16 {
    type MembranePotentialQuant = bf16;
}
#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMExclusivePowerQuantFloat32;

impl NMExclusivePowerTrait for NMExclusivePowerQuantFloat32 {}

impl NeuronModel for NMExclusivePowerQuantFloat32 {
    type NeuronLayoutTag = NeuronLayoutTagVoxel;

    type NeurogenesisScratchVarianceDefaults<FIQ: FeagiIndexQuantization> = super::neurogenesis::ExclusivePowerNeurogenesisDefaults<FIQ, Self>;

    type NeurogenesisByGenerator<FIQ: FeagiIndexQuantization> = NeurogenesisGeneratorUniformDefault<FIQ, Self>;

    type CorticalDataProperties = NMExclusivePowerDataCorticalProperties<Self>;
    type CorticalDataInternal = NMExclusivePowerDataCorticalInternal<Self>;
    type CorticalDataScratch = NMExclusivePowerDataCorticalScratch<Self>;
    type CorticalDataVariance = NMExclusivePowerDataCorticalVariance<Self>;
    type NeuronDataInternal = NMExclusivePowerDataNeuronInternal<Self>;
    type NeuronDataScratch = NMExclusivePowerDataNeuronScratch<Self>;
    type NeuronDataVariance = NMExclusivePowerDataNeuronVariance<Self>;

    type QuantKind = NMExclusivePowerQuantKind;
    const QUANT_KIND: Self::QuantKind = NMExclusivePowerQuantKind::Float32;
}

impl MembranePotentialQuantization for NMExclusivePowerQuantFloat32 {
    type MembranePotentialQuant = f32;
}

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMExclusivePowerQuantFloat64;

impl NMExclusivePowerTrait for NMExclusivePowerQuantFloat64 {}

impl NeuronModel for NMExclusivePowerQuantFloat64 {
    type NeuronLayoutTag = NeuronLayoutTagVoxel;

    type NeurogenesisScratchVarianceDefaults<FIQ: FeagiIndexQuantization> = super::neurogenesis::ExclusivePowerNeurogenesisDefaults<FIQ, Self>;

    type NeurogenesisByGenerator<FIQ: FeagiIndexQuantization> = NeurogenesisGeneratorUniformDefault<FIQ, Self>;

    type CorticalDataProperties = NMExclusivePowerDataCorticalProperties<Self>;
    type CorticalDataInternal = NMExclusivePowerDataCorticalInternal<Self>;
    type CorticalDataScratch = NMExclusivePowerDataCorticalScratch<Self>;
    type CorticalDataVariance = NMExclusivePowerDataCorticalVariance<Self>;
    type NeuronDataInternal = NMExclusivePowerDataNeuronInternal<Self>;
    type NeuronDataScratch = NMExclusivePowerDataNeuronScratch<Self>;
    type NeuronDataVariance = NMExclusivePowerDataNeuronVariance<Self>;

    type QuantKind = NMExclusivePowerQuantKind;
    const QUANT_KIND: Self::QuantKind = NMExclusivePowerQuantKind::Float64;
}

impl MembranePotentialQuantization for NMExclusivePowerQuantFloat64 {
    type MembranePotentialQuant = f64;
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
            NMKind::ExclusivePower(v) => Ok(v),
            _ => Err(NeuronModelEnumConversionError::IncompatibleModel),
        }
    }
}

impl<C: NMContainer> From<NMExclusivePowerQuanted<C>> for NMModeled<C> {
    fn from(value: NMExclusivePowerQuanted<C>) -> Self {
        NMModeled::ExclusivePower(value)
    }
}

impl<C: NMContainer> TryFrom<NMModeled<C>> for NMExclusivePowerQuanted<C> {
    type Error = NeuronModelEnumConversionError<NMKind>;

    fn try_from(value: NMModeled<C>) -> Result<Self, Self::Error> {
        match value {
            NMModeled::ExclusivePower(v) => Ok(v),
            _ => Err(NeuronModelEnumConversionError::IncompatibleModel),
        }
    }
}

//endregion
