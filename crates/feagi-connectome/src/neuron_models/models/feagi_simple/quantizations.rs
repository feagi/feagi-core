use crate::neuron_models::models::kind::NMKind;
use crate::neuron_models::models::{NMContainer, NMMember, NMModeled, NeuronModelEnumConversionError};
use quant_standard_32_bit::NMFeagiSimpleQuantStandard32Bit;


pub mod quant_standard_32_bit {
    use crate::feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::voxel::NeuronLayoutTagVoxel;
    use crate::feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
    use crate::feagi_connectome_neural::neural_basis::quantization_levels::{FeagiIndexQuantization, MembranePotentialQuantization};
    use crate::neuron_models::models::feagi_simple::{NMFeagiSimpleDataCorticalInternal, NMFeagiSimpleDataCorticalProperties, NMFeagiSimpleDataCorticalScratch, NMFeagiSimpleDataCorticalVariance, NMFeagiSimpleDataNeuronInternal, NMFeagiSimpleDataNeuronScratch, NMFeagiSimpleDataNeuronVariance, NMFeagiSimpleTrait};
    use crate::neuron_models::models::feagi_simple::neurogenesis::FeagiSimpleScratchVarianceDefaults;
    use crate::neuron_models::models::feagi_simple::quantizations::NMFeagiSimpleQuantKind;
    use crate::neuron_models::neurogenesis_generator_uniform_defaults::NeurogenesisGeneratorUniformDefault;

    #[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
    pub struct NMFeagiSimpleQuantStandard32Bit;

    impl NMFeagiSimpleTrait for NMFeagiSimpleQuantStandard32Bit {
        type Period = f32;
    }

    impl NeuronModel for NMFeagiSimpleQuantStandard32Bit {
        type NeuronLayoutTag = NeuronLayoutTagVoxel;

        type NeurogenesisScratchVarianceDefaults<FIQ: FeagiIndexQuantization> = FeagiSimpleScratchVarianceDefaults<FIQ, Self>;

        type NeurogenesisByGenerator<FIQ: FeagiIndexQuantization> = NeurogenesisGeneratorUniformDefault<FIQ, Self>;

        type CorticalDataProperties = NMFeagiSimpleDataCorticalProperties<Self>;
        type CorticalDataInternal = NMFeagiSimpleDataCorticalInternal<Self>;
        type CorticalDataScratch = NMFeagiSimpleDataCorticalScratch<Self>;
        type CorticalDataVariance = NMFeagiSimpleDataCorticalVariance<Self>;
        type NeuronDataInternal = NMFeagiSimpleDataNeuronInternal<Self>;
        type NeuronDataScratch = NMFeagiSimpleDataNeuronScratch<Self>;
        type NeuronDataVariance = NMFeagiSimpleDataNeuronVariance<Self>;

        type QuantKind = NMFeagiSimpleQuantKind;

        const QUANT_KIND: Self::QuantKind = NMFeagiSimpleQuantKind::Standard32Bit;
    }

    impl MembranePotentialQuantization for NMFeagiSimpleQuantStandard32Bit {
        type MembranePotentialQuant = f32;
    }
}

//region Shared

//region Quanted

/// Data of container `C` for a given quantization of the Feagi Simple Neuron Model
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(bound(
    serialize = "C::Of<NMFeagiSimpleQuantStandard32Bit>: ::serde::Serialize",
    deserialize = "C::Of<NMFeagiSimpleQuantStandard32Bit>: ::serde::Deserialize<'de>"
))]
pub enum NMFeagiSimpleQuanted<C: NMContainer> {
    Standard32Bit(C::Of<NMFeagiSimpleQuantStandard32Bit>),
}

impl<C: NMContainer> NMFeagiSimpleQuanted<C> {
    /// The quantization of the data held
    pub const fn quant_kind(&self) -> NMFeagiSimpleQuantKind {
        match self {
            Self::Standard32Bit(_) => NMFeagiSimpleQuantKind::Standard32Bit,
        }
    }
}

impl<C: NMContainer> Clone for NMFeagiSimpleQuanted<C>
where
    C::Of<NMFeagiSimpleQuantStandard32Bit>: Clone,
{
    fn clone(&self) -> Self {
        match self {
            Self::Standard32Bit(v) => Self::Standard32Bit(v.clone()),
        }
    }
}

impl<C: NMContainer> core::fmt::Debug for NMFeagiSimpleQuanted<C>
where
    C::Of<NMFeagiSimpleQuantStandard32Bit>: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Standard32Bit(v) => f.debug_tuple("Standard32Bit").field(v).finish(),
        }
    }
}

impl NMMember for NMFeagiSimpleQuantStandard32Bit {
    type Quanted<C: NMContainer> = NMFeagiSimpleQuanted<C>;

    fn wrap<C: NMContainer>(data: C::Of<Self>) -> NMFeagiSimpleQuanted<C> {
        NMFeagiSimpleQuanted::Standard32Bit(data)
    }

    fn unwrap<C: NMContainer>(quanted: NMFeagiSimpleQuanted<C>) -> Result<C::Of<Self>, NeuronModelEnumConversionError<NMFeagiSimpleQuantKind>> {
        match quanted {
            NMFeagiSimpleQuanted::Standard32Bit(v) => Ok(v),
        }
    }
}

//endregion

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, ::serde::Serialize, ::serde::Deserialize)]
/// Describes the quantization of the Feagi Simple Neuron Model
pub enum NMFeagiSimpleQuantKind {
    Standard32Bit,
}


impl From<NMFeagiSimpleQuantKind> for NMKind {
    fn from(value: NMFeagiSimpleQuantKind) -> Self {
        NMKind::FeagiSimple(value)
    }
}

impl TryFrom<NMKind> for NMFeagiSimpleQuantKind {
    type Error = NeuronModelEnumConversionError<NMKind>;

    fn try_from(value: NMKind) -> Result<Self, Self::Error> {
        match value {
            NMKind::FeagiSimple(v) => Ok(v),
            _ => Err(NeuronModelEnumConversionError::IncompatibleModel),
        }
    }
}

impl<C: NMContainer> From<NMFeagiSimpleQuanted<C>> for NMModeled<C> {
    fn from(value: NMFeagiSimpleQuanted<C>) -> Self {
        NMModeled::FeagiSimple(value)
    }
}

impl<C: NMContainer> TryFrom<NMModeled<C>> for NMFeagiSimpleQuanted<C> {
    type Error = NeuronModelEnumConversionError<NMKind>;

    fn try_from(value: NMModeled<C>) -> Result<Self, Self::Error> {
        match value {
            NMModeled::FeagiSimple(v) => Ok(v),
            _ => Err(NeuronModelEnumConversionError::IncompatibleModel),
        }
    }
}

//endregion
