use crate::neuron_models::models::kind::NMKind;
use crate::neuron_models::models::{NMContainer, NMMember, NMModeled, NeuronModelEnumConversionError};
use quant_standard_32_bit::NMFeagiAdvancedQuantStandard32Bit;


pub mod quant_standard_32_bit {
    use crate::feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::voxel::NeuronLayoutTagVoxel;
    use crate::feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
    use crate::feagi_connectome_neural::neural_basis::quantization_levels::{FeagiIndexQuantization, MembranePotentialQuantization};
    use crate::neuron_models::models::feagi_advanced::{NMFeagiAdvancedDataCorticalInternal, NMFeagiAdvancedDataCorticalProperties, NMFeagiAdvancedDataCorticalScratch, NMFeagiAdvancedDataCorticalVariance, NMFeagiAdvancedDataNeuronInternal, NMFeagiAdvancedDataNeuronScratch, NMFeagiAdvancedDataNeuronVariance, NMFeagiAdvancedTrait};
    use crate::neuron_models::models::feagi_advanced::neurogenesis::FeagiAdvancedScratchVarianceDefaults;
    use crate::neuron_models::models::feagi_advanced::quantizations::NMFeagiAdvancedQuantKind;
    use crate::neuron_models::neurogenesis_generator_uniform_defaults::NeurogenesisGeneratorUniformDefault;

    #[derive(Clone, Copy, PartialEq, Debug, Eq, Hash, ::serde::Serialize, ::serde::Deserialize)]
    pub struct NMFeagiAdvancedQuantStandard32Bit;

    impl NMFeagiAdvancedTrait for NMFeagiAdvancedQuantStandard32Bit {
        type Period = f32;
    }

    impl NeuronModel for NMFeagiAdvancedQuantStandard32Bit {
        type NeuronLayoutTag = NeuronLayoutTagVoxel;

        type NeurogenesisScratchVarianceDefaults<FIQ: FeagiIndexQuantization> = FeagiAdvancedScratchVarianceDefaults<FIQ, Self>;

        type NeurogenesisByGenerator<FIQ: FeagiIndexQuantization> = NeurogenesisGeneratorUniformDefault<FIQ, Self>;

        type CorticalDataProperties = NMFeagiAdvancedDataCorticalProperties<Self>;
        type CorticalDataInternal = NMFeagiAdvancedDataCorticalInternal<Self>;
        type CorticalDataScratch = NMFeagiAdvancedDataCorticalScratch<Self>;
        type CorticalDataVariance = NMFeagiAdvancedDataCorticalVariance<Self>;
        type NeuronDataInternal = NMFeagiAdvancedDataNeuronInternal<Self>;
        type NeuronDataScratch = NMFeagiAdvancedDataNeuronScratch<Self>;
        type NeuronDataVariance = NMFeagiAdvancedDataNeuronVariance<Self>;

        type QuantKind = NMFeagiAdvancedQuantKind;
        const QUANT_KIND: Self::QuantKind = NMFeagiAdvancedQuantKind::Standard32Bit(Self);
    }

    impl MembranePotentialQuantization for NMFeagiAdvancedQuantStandard32Bit {
        type MembranePotentialQuant = f32;
    }
}

//region Shared

//region Quanted

/// Data of container `C` for a given quantization of the Feagi Advanced Neuron Model
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(bound(
    serialize = "C::Of<NMFeagiAdvancedQuantStandard32Bit>: ::serde::Serialize",
    deserialize = "C::Of<NMFeagiAdvancedQuantStandard32Bit>: ::serde::Deserialize<'de>"
))]
pub enum NMFeagiAdvancedQuanted<C: NMContainer> {
    Standard32Bit(C::Of<NMFeagiAdvancedQuantStandard32Bit>),
}

impl<C: NMContainer> NMFeagiAdvancedQuanted<C> {
    /// The quantization of the data held
    pub const fn quant_kind(&self) -> NMFeagiAdvancedQuantKind {
        match self {
            Self::Standard32Bit(_) => NMFeagiAdvancedQuantKind::Standard32Bit(NMFeagiAdvancedQuantStandard32Bit),
        }
    }
}

impl<C: NMContainer> Clone for NMFeagiAdvancedQuanted<C>
where
    C::Of<NMFeagiAdvancedQuantStandard32Bit>: Clone,
{
    fn clone(&self) -> Self {
        match self {
            Self::Standard32Bit(v) => Self::Standard32Bit(v.clone()),
        }
    }
}

impl<C: NMContainer> core::fmt::Debug for NMFeagiAdvancedQuanted<C>
where
    C::Of<NMFeagiAdvancedQuantStandard32Bit>: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Standard32Bit(v) => f.debug_tuple("Standard32Bit").field(v).finish(),
        }
    }
}

impl NMMember for NMFeagiAdvancedQuantStandard32Bit {
    type Quanted<C: NMContainer> = NMFeagiAdvancedQuanted<C>;

    fn wrap<C: NMContainer>(data: C::Of<Self>) -> NMFeagiAdvancedQuanted<C> {
        NMFeagiAdvancedQuanted::Standard32Bit(data)
    }

    fn unwrap<C: NMContainer>(quanted: NMFeagiAdvancedQuanted<C>) -> Result<C::Of<Self>, NeuronModelEnumConversionError<NMFeagiAdvancedQuantKind>> {
        match quanted {
            NMFeagiAdvancedQuanted::Standard32Bit(v) => Ok(v),
        }
    }
}

//endregion

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, ::serde::Serialize, ::serde::Deserialize)]
/// Describes the quantization of the Feagi Advanced Neuron Model
pub enum NMFeagiAdvancedQuantKind {
    Standard32Bit(NMFeagiAdvancedQuantStandard32Bit),
}


impl From<NMFeagiAdvancedQuantKind> for NMKind {
    fn from(value: NMFeagiAdvancedQuantKind) -> Self {
        NMKind::FeagiAdvanced(value)
    }
}

impl TryFrom<NMKind> for NMFeagiAdvancedQuantKind {
    type Error = NeuronModelEnumConversionError<NMKind>;

    fn try_from(value: NMKind) -> Result<Self, Self::Error> {
        match value {
            NMKind::FeagiAdvanced(v) => Ok(v),
            _ => Err(NeuronModelEnumConversionError::IncompatibleModel),
        }
    }
}

impl<C: NMContainer> From<NMFeagiAdvancedQuanted<C>> for NMModeled<C> {
    fn from(value: NMFeagiAdvancedQuanted<C>) -> Self {
        NMModeled::FeagiAdvanced(value)
    }
}

impl<C: NMContainer> TryFrom<NMModeled<C>> for NMFeagiAdvancedQuanted<C> {
    type Error = NeuronModelEnumConversionError<NMKind>;

    fn try_from(value: NMModeled<C>) -> Result<Self, Self::Error> {
        match value {
            NMModeled::FeagiAdvanced(v) => Ok(v),
            _ => Err(NeuronModelEnumConversionError::IncompatibleModel),
        }
    }
}

//endregion
