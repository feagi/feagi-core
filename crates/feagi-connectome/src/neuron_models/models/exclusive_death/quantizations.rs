use crate::neuron_models::models::exclusive_death::{NMExclusiveDeathDataCorticalInternal, NMExclusiveDeathDataCorticalProperties, NMExclusiveDeathDataCorticalScratch, NMExclusiveDeathDataCorticalVariance, NMExclusiveDeathDataNeuronInternal, NMExclusiveDeathDataNeuronScratch, NMExclusiveDeathDataNeuronVariance, NMExclusiveDeathTrait};
use crate::neuron_models::models::kind::{NMKind};
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::voxel::NeuronLayoutTagVoxel;
use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome_neural::neural_basis::quantization_levels::{FeagiIndexQuantization, MembranePotentialQuantization};
use half::{bf16, f16};
use crate::neuron_models::neurogenesis_generator_uniform_defaults::NeurogenesisGeneratorUniformDefault;
use crate::neuron_models::models::{NMContainer, NMMember, NMModeled, NeuronModelEnumConversionError};

//region enums

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
/// Describes the quantization of the Exclusive Death Neuron Model
pub enum NMExclusiveDeathQuantKind {
    Float16,
    BFloat16,
    Float32,
    Float64,
}
//region Quanted

/// Data of container `C` for a given quantization of the Exclusive Death Neuron Model
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(bound(serialize = "C::Of<NMExclusiveDeathQuantFloat16>: ::serde::Serialize, C::Of<NMExclusiveDeathQuantBFloat16>: ::serde::Serialize, C::Of<NMExclusiveDeathQuantFloat32>: ::serde::Serialize, C::Of<NMExclusiveDeathQuantFloat64>: ::serde::Serialize", deserialize = "C::Of<NMExclusiveDeathQuantFloat16>: ::serde::Deserialize<'de>, C::Of<NMExclusiveDeathQuantBFloat16>: ::serde::Deserialize<'de>, C::Of<NMExclusiveDeathQuantFloat32>: ::serde::Deserialize<'de>, C::Of<NMExclusiveDeathQuantFloat64>: ::serde::Deserialize<'de>"))]
pub enum NMExclusiveDeathQuanted<C: NMContainer> {
    Float16(C::Of<NMExclusiveDeathQuantFloat16>),
    BFloat16(C::Of<NMExclusiveDeathQuantBFloat16>),
    Float32(C::Of<NMExclusiveDeathQuantFloat32>),
    Float64(C::Of<NMExclusiveDeathQuantFloat64>),
}

impl<C: NMContainer> NMExclusiveDeathQuanted<C> {
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

impl<C: NMContainer> Clone for NMExclusiveDeathQuanted<C>
where
    C::Of<NMExclusiveDeathQuantFloat16>: Clone,
    C::Of<NMExclusiveDeathQuantBFloat16>: Clone,
    C::Of<NMExclusiveDeathQuantFloat32>: Clone,
    C::Of<NMExclusiveDeathQuantFloat64>: Clone,
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

impl<C: NMContainer> core::fmt::Debug for NMExclusiveDeathQuanted<C>
where
    C::Of<NMExclusiveDeathQuantFloat16>: core::fmt::Debug,
    C::Of<NMExclusiveDeathQuantBFloat16>: core::fmt::Debug,
    C::Of<NMExclusiveDeathQuantFloat32>: core::fmt::Debug,
    C::Of<NMExclusiveDeathQuantFloat64>: core::fmt::Debug,
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

impl NMMember for NMExclusiveDeathQuantFloat16 {
    type Quanted<C: NMContainer> = NMExclusiveDeathQuanted<C>;

    fn wrap<C: NMContainer>(data: C::Of<Self>) -> NMExclusiveDeathQuanted<C> {
        NMExclusiveDeathQuanted::Float16(data)
    }

    fn unwrap<C: NMContainer>(quanted: NMExclusiveDeathQuanted<C>) -> Result<C::Of<Self>, NeuronModelEnumConversionError<NMExclusiveDeathQuantKind>> {
        match quanted {
            NMExclusiveDeathQuanted::Float16(v) => Ok(v),
            other => Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                expected: Self::QUANT_KIND,
                actual: other.quant_kind(),
            }),
        }
    }
}

impl NMMember for NMExclusiveDeathQuantBFloat16 {
    type Quanted<C: NMContainer> = NMExclusiveDeathQuanted<C>;

    fn wrap<C: NMContainer>(data: C::Of<Self>) -> NMExclusiveDeathQuanted<C> {
        NMExclusiveDeathQuanted::BFloat16(data)
    }

    fn unwrap<C: NMContainer>(quanted: NMExclusiveDeathQuanted<C>) -> Result<C::Of<Self>, NeuronModelEnumConversionError<NMExclusiveDeathQuantKind>> {
        match quanted {
            NMExclusiveDeathQuanted::BFloat16(v) => Ok(v),
            other => Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                expected: Self::QUANT_KIND,
                actual: other.quant_kind(),
            }),
        }
    }
}

impl NMMember for NMExclusiveDeathQuantFloat32 {
    type Quanted<C: NMContainer> = NMExclusiveDeathQuanted<C>;

    fn wrap<C: NMContainer>(data: C::Of<Self>) -> NMExclusiveDeathQuanted<C> {
        NMExclusiveDeathQuanted::Float32(data)
    }

    fn unwrap<C: NMContainer>(quanted: NMExclusiveDeathQuanted<C>) -> Result<C::Of<Self>, NeuronModelEnumConversionError<NMExclusiveDeathQuantKind>> {
        match quanted {
            NMExclusiveDeathQuanted::Float32(v) => Ok(v),
            other => Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                expected: Self::QUANT_KIND,
                actual: other.quant_kind(),
            }),
        }
    }
}

impl NMMember for NMExclusiveDeathQuantFloat64 {
    type Quanted<C: NMContainer> = NMExclusiveDeathQuanted<C>;

    fn wrap<C: NMContainer>(data: C::Of<Self>) -> NMExclusiveDeathQuanted<C> {
        NMExclusiveDeathQuanted::Float64(data)
    }

    fn unwrap<C: NMContainer>(quanted: NMExclusiveDeathQuanted<C>) -> Result<C::Of<Self>, NeuronModelEnumConversionError<NMExclusiveDeathQuantKind>> {
        match quanted {
            NMExclusiveDeathQuanted::Float64(v) => Ok(v),
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
    const QUANT_KIND: Self::QuantKind = NMExclusiveDeathQuantKind::Float16;
}

impl MembranePotentialQuantization for NMExclusiveDeathQuantFloat16
{
    type MembranePotentialQuant = f16;
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
    const QUANT_KIND: Self::QuantKind = NMExclusiveDeathQuantKind::BFloat16;
}

impl MembranePotentialQuantization for NMExclusiveDeathQuantBFloat16
{
    type MembranePotentialQuant = bf16;
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
    const QUANT_KIND: Self::QuantKind = NMExclusiveDeathQuantKind::Float32;
}

impl MembranePotentialQuantization for NMExclusiveDeathQuantFloat32
{
    type MembranePotentialQuant = f32;
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
    const QUANT_KIND: Self::QuantKind = NMExclusiveDeathQuantKind::Float64;
}

impl MembranePotentialQuantization for NMExclusiveDeathQuantFloat64
{
    type MembranePotentialQuant = f64;
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




impl<C: NMContainer> From<NMExclusiveDeathQuanted<C>> for NMModeled<C> {
    fn from(value: NMExclusiveDeathQuanted<C>) -> Self {
        NMModeled::ExclusiveDeath(value)
    }
}

impl<C: NMContainer> TryFrom<NMModeled<C>> for NMExclusiveDeathQuanted<C> {
    type Error = NeuronModelEnumConversionError<NMKind>;

    fn try_from(value: NMModeled<C>) -> Result<Self, Self::Error> {
        match value {
            NMModeled::ExclusiveDeath(v) => Ok(v),
            _ => Err(NeuronModelEnumConversionError::IncompatibleModel),
        }
    }
}

//endregion
