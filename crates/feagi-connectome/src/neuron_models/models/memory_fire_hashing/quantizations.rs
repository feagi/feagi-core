use crate::neuron_models::models::memory_fire_hashing::{NMMemoryFireHashingDataCorticalInternal, NMMemoryFireHashingDataCorticalProperties, NMMemoryFireHashingDataCorticalScratch, NMMemoryFireHashingDataCorticalVariance, NMMemoryFireHashingDataNeuronInternal, NMMemoryFireHashingDataNeuronScratch, NMMemoryFireHashingDataNeuronVariance, NMMemoryFireHashingTrait};
use crate::neuron_models::models::kind::{NMKind};
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::single_stepped_dynamic::NeuronLayoutTagSingleSteppedDynamic;
use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome_neural::neural_basis::quantization_levels::{FeagiIndexQuantization, MembranePotentialQuantization};
use crate::neuron_models::neurogenesis_generator_uniform_defaults::NeurogenesisGeneratorUniformDefault;
use crate::neuron_models::models::{NMContainer, NMMember, NMModeled, NeuronModelEnumConversionError};

//region enums

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
/// Describes the quantization of the Memory Fire Hashing Neuron Model
pub enum NMMemoryFireHashingQuantKind {
    Standard32Bit,
}


//region Quanted

/// Data of container `C` for a given quantization of the Memory Fire Hashing Neuron Model
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(bound(serialize = "C::Of<NMMemoryFireHashingQuantStandard32Bit>: ::serde::Serialize", deserialize = "C::Of<NMMemoryFireHashingQuantStandard32Bit>: ::serde::Deserialize<'de>"))]
pub enum NMMemoryFireHashingQuanted<C: NMContainer> {
    Standard32Bit(C::Of<NMMemoryFireHashingQuantStandard32Bit>),
}

impl<C: NMContainer> NMMemoryFireHashingQuanted<C> {
    /// The quantization of the data held
    pub const fn quant_kind(&self) -> NMMemoryFireHashingQuantKind {
        match self {
            Self::Standard32Bit(_) => NMMemoryFireHashingQuantKind::Standard32Bit,
        }
    }
}

impl<C: NMContainer> Clone for NMMemoryFireHashingQuanted<C>
where
    C::Of<NMMemoryFireHashingQuantStandard32Bit>: Clone,
{
    fn clone(&self) -> Self {
        match self {
            Self::Standard32Bit(v) => Self::Standard32Bit(v.clone()),
        }
    }
}

impl<C: NMContainer> core::fmt::Debug for NMMemoryFireHashingQuanted<C>
where
    C::Of<NMMemoryFireHashingQuantStandard32Bit>: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Standard32Bit(v) => f.debug_tuple("Standard32Bit").field(v).finish(),
        }
    }
}

impl NMMember for NMMemoryFireHashingQuantStandard32Bit {
    type Quanted<C: NMContainer> = NMMemoryFireHashingQuanted<C>;

    fn wrap<C: NMContainer>(data: C::Of<Self>) -> NMMemoryFireHashingQuanted<C> {
        NMMemoryFireHashingQuanted::Standard32Bit(data)
    }

    fn unwrap<C: NMContainer>(quanted: NMMemoryFireHashingQuanted<C>) -> Result<C::Of<Self>, NeuronModelEnumConversionError<NMMemoryFireHashingQuantKind>> {
        match quanted {
            NMMemoryFireHashingQuanted::Standard32Bit(v) => Ok(v),
        }
    }
}

//endregion


//endregion

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMMemoryFireHashingQuantStandard32Bit;

impl NMMemoryFireHashingTrait for NMMemoryFireHashingQuantStandard32Bit {
    type Period = f32;
}

impl NeuronModel for NMMemoryFireHashingQuantStandard32Bit {
    type NeuronLayoutTag = NeuronLayoutTagSingleSteppedDynamic;

    type NeurogenesisScratchVarianceDefaults<FIQ: FeagiIndexQuantization> =
        super::neurogenesis::MemoryFireHashingNeurogenesisDefaults<FIQ, Self>;

    type NeurogenesisByGenerator<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorUniformDefault<FIQ, Self>;

    type CorticalDataProperties = NMMemoryFireHashingDataCorticalProperties<Self>;
    type CorticalDataInternal = NMMemoryFireHashingDataCorticalInternal<Self>;
    type CorticalDataScratch = NMMemoryFireHashingDataCorticalScratch<Self>;
    type CorticalDataVariance = NMMemoryFireHashingDataCorticalVariance<Self>;
    type NeuronDataInternal = NMMemoryFireHashingDataNeuronInternal<Self>;
    type NeuronDataScratch = NMMemoryFireHashingDataNeuronScratch<Self>;
    type NeuronDataVariance = NMMemoryFireHashingDataNeuronVariance<Self>;

    type QuantKind = NMMemoryFireHashingQuantKind;
    const QUANT_KIND: Self::QuantKind = NMMemoryFireHashingQuantKind::Standard32Bit;
}

impl MembranePotentialQuantization for NMMemoryFireHashingQuantStandard32Bit
{
    type MembranePotentialQuant = f32;
}



//region Model Level Conversions

impl From<NMMemoryFireHashingQuantKind> for NMKind {
    fn from(value: NMMemoryFireHashingQuantKind) -> Self {
        NMKind::MemoryFireHashing(value)
    }
}

impl TryFrom<NMKind> for NMMemoryFireHashingQuantKind {
    type Error = NeuronModelEnumConversionError<NMKind>;

    fn try_from(value: NMKind) -> Result<Self, Self::Error> {
        match value {
            NMKind::MemoryFireHashing(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleModel)
            }
        }
    }
}


impl<C: NMContainer> From<NMMemoryFireHashingQuanted<C>> for NMModeled<C> {
    fn from(value: NMMemoryFireHashingQuanted<C>) -> Self {
        NMModeled::MemoryFireHashing(value)
    }
}

impl<C: NMContainer> TryFrom<NMModeled<C>> for NMMemoryFireHashingQuanted<C> {
    type Error = NeuronModelEnumConversionError<NMKind>;

    fn try_from(value: NMModeled<C>) -> Result<Self, Self::Error> {
        match value {
            NMModeled::MemoryFireHashing(v) => Ok(v),
            _ => Err(NeuronModelEnumConversionError::IncompatibleModel),
        }
    }
}

//endregion
