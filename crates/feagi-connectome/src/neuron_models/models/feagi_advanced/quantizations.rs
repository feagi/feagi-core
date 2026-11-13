use crate::neuron_models::models::feagi_advanced::{NMFeagiAdvancedDataCorticalInternal, NMFeagiAdvancedDataCorticalProperties, NMFeagiAdvancedDataCorticalScratch, NMFeagiAdvancedDataCorticalVariance, NMFeagiAdvancedDataNeuronInternal, NMFeagiAdvancedDataNeuronScratch, NMFeagiAdvancedDataNeuronVariance, NMFeagiAdvancedTrait};
use crate::neuron_models::models::kind::NMKind;
use crate::neuron_models::models::container::{NMAccessError, NMContainer, NMContainerMarker, NMMember, NMVisitor, NMVisitorMut, NMVisitorRef};
use crate::neuron_models::models::NMModeled;
use crate::neuron_models::neurogenesis_generator_uniform_defaults::{NeurogenesisGeneratorPrecursorUniformDefault, NeurogenesisGeneratorUniformDefault};
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::voxel::NeuronLayoutTagVoxel;
use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome_neural::neural_basis::quantization_levels::{FeagiIndexQuantization, MembranePotentialQuantization};


//region enums

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
/// Describes the quantization of the Feagi Advanced Neuron Model
pub enum NMFeagiAdvancedQuantKind {
    Standard32Bit,
}

/// The data of container `C` for the FeagiAdvanced Neuron Model, with a variant per quantization
#[derive(Clone, Debug, ::serde::Serialize, ::serde::Deserialize)]
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
            Self::Standard32Bit(_) => NMFeagiAdvancedQuantKind::Standard32Bit,
        }
    }

    /// The model and quantization of the data held
    pub const fn kind(&self) -> NMKind {
        NMKind::FeagiAdvanced(self.quant_kind())
    }

    /// Runs the visitor on the data held, with the member it is held for
    pub fn dispatch<V: NMVisitor<C>>(self, visitor: V) -> V::Output {
        match self {
            Self::Standard32Bit(data) => visitor.visit::<NMFeagiAdvancedQuantStandard32Bit>(data),
        }
    }

    /// Runs the visitor on a reference to the data held, with the member it is held for
    pub fn dispatch_ref<'a, V: NMVisitorRef<'a, C>>(&'a self, visitor: V) -> V::Output {
        match self {
            Self::Standard32Bit(data) => visitor.visit::<NMFeagiAdvancedQuantStandard32Bit>(data),
        }
    }

    /// Runs the visitor on a mutable reference to the data held, with the member it is held for
    pub fn dispatch_mut<'a, V: NMVisitorMut<'a, C>>(&'a mut self, visitor: V) -> V::Output {
        match self {
            Self::Standard32Bit(data) => visitor.visit::<NMFeagiAdvancedQuantStandard32Bit>(data),
        }
    }
}

/// The marker struct of a quantization of the FeagiAdvanced Neuron Model
pub type NMFeagiAdvancedQuantedKind = NMFeagiAdvancedQuanted<NMContainerMarker>;

impl NMFeagiAdvancedQuantedKind {
    /// The marker struct of quantization `NM` of this model
    pub fn from_nm_quant<NM: NMFeagiAdvancedTrait>() -> Self {
        match NM::QUANT_KIND {
            NMFeagiAdvancedQuantKind::Standard32Bit => Self::Standard32Bit(NMFeagiAdvancedQuantStandard32Bit),
        }
    }
}

// The marker structs are unit structs, so the kind is fully described by its quantization
impl Copy for NMFeagiAdvancedQuantedKind {}

impl PartialEq for NMFeagiAdvancedQuantedKind {
    fn eq(&self, other: &Self) -> bool {
        self.quant_kind() == other.quant_kind()
    }
}

impl Eq for NMFeagiAdvancedQuantedKind {}

//endregion

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMFeagiAdvancedQuantStandard32Bit;

impl NMFeagiAdvancedTrait for NMFeagiAdvancedQuantStandard32Bit {
    type Period = f32;
}

impl NeuronModel for NMFeagiAdvancedQuantStandard32Bit {
    type NeuronLayoutTag = NeuronLayoutTagVoxel;

    type NeurogenesisScratchVarianceGeneratorType<FIQ: FeagiIndexQuantization> =
        super::neurogenesis::FeagiAdvancedScratchVarianceGenerator<FIQ, Self>;

    type NeurogenesisGenerator<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorUniformDefault<FIQ, Self>;
    type NeurogenesisGeneratorPrecursor<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorPrecursorUniformDefault<FIQ, Self>;

    type CorticalDataProperties = NMFeagiAdvancedDataCorticalProperties<Self>;
    type CorticalDataInternal = NMFeagiAdvancedDataCorticalInternal<Self>;
    type CorticalDataScratch = NMFeagiAdvancedDataCorticalScratch<Self>;
    type CorticalDataVariance = NMFeagiAdvancedDataCorticalVariance<Self>;
    type NeuronDataInternal = NMFeagiAdvancedDataNeuronInternal<Self>;
    type NeuronDataScratch = NMFeagiAdvancedDataNeuronScratch<Self>;
    type NeuronDataVariance = NMFeagiAdvancedDataNeuronVariance<Self>;
    
    type QuantKind = NMFeagiAdvancedQuantKind;
    type QuantedKind = NMFeagiAdvancedQuantedKind;
    const QUANT_KIND: Self::QuantKind = NMFeagiAdvancedQuantKind::Standard32Bit;
    const QUANTED_KIND: Self::QuantedKind = NMFeagiAdvancedQuantedKind::Standard32Bit(Self);
}

impl MembranePotentialQuantization for NMFeagiAdvancedQuantStandard32Bit
{
    type MembranePotentialQuant = f32;
}

impl From<NMFeagiAdvancedQuantStandard32Bit> for NMFeagiAdvancedQuantedKind {
    fn from(value: NMFeagiAdvancedQuantStandard32Bit) -> Self {
        NMFeagiAdvancedQuantedKind::Standard32Bit(value)
    }
}

impl TryFrom<NMFeagiAdvancedQuantedKind> for NMFeagiAdvancedQuantStandard32Bit {
    type Error = NMAccessError;

    fn try_from(value: NMFeagiAdvancedQuantedKind) -> Result<Self, Self::Error> {
        match value {
            NMFeagiAdvancedQuanted::Standard32Bit(marker) => Ok(marker),
            #[allow(unreachable_patterns)]
            other => Err(NMAccessError::IncorrectMember { expected: <Self as NMMember>::KIND, actual: other.kind() }),
        }
    }
}

impl NMMember for NMFeagiAdvancedQuantStandard32Bit {
    const KIND: NMKind = NMKind::FeagiAdvanced(NMFeagiAdvancedQuantKind::Standard32Bit);
    const MARKER: Self = Self;

    fn into_modeled<C: NMContainer>(data: C::Of<Self>) -> NMModeled<C> {
        NMModeled::FeagiAdvanced(NMFeagiAdvancedQuanted::Standard32Bit(data))
    }

    fn try_from_modeled<C: NMContainer>(value: NMModeled<C>) -> Result<C::Of<Self>, NMAccessError> {
        let actual = value.kind();
        match value {
            NMModeled::FeagiAdvanced(NMFeagiAdvancedQuanted::Standard32Bit(data)) => Ok(data),
            _ => Err(NMAccessError::IncorrectMember { expected: Self::KIND, actual }),
        }
    }

    fn try_ref_modeled<C: NMContainer>(value: &NMModeled<C>) -> Result<&C::Of<Self>, NMAccessError> {
        match value {
            NMModeled::FeagiAdvanced(NMFeagiAdvancedQuanted::Standard32Bit(data)) => Ok(data),
            _ => Err(NMAccessError::IncorrectMember { expected: Self::KIND, actual: value.kind() }),
        }
    }

    fn try_mut_modeled<C: NMContainer>(value: &mut NMModeled<C>) -> Result<&mut C::Of<Self>, NMAccessError> {
        let actual = value.kind();
        match value {
            NMModeled::FeagiAdvanced(NMFeagiAdvancedQuanted::Standard32Bit(data)) => Ok(data),
            _ => Err(NMAccessError::IncorrectMember { expected: Self::KIND, actual }),
        }
    }
}

//region Model Level Conversions

impl From<NMFeagiAdvancedQuantKind> for NMKind {
    fn from(value: NMFeagiAdvancedQuantKind) -> Self {
        NMKind::FeagiAdvanced(value)
    }
}

impl TryFrom<NMKind> for NMFeagiAdvancedQuantKind {
    type Error = NMAccessError;

    fn try_from(value: NMKind) -> Result<Self, Self::Error> {
        match value {
            NMKind::FeagiAdvanced(quant) => Ok(quant),
            actual => Err(NMAccessError::IncorrectModel { expected: "FeagiAdvanced", actual }),
        }
    }
}

impl<C: NMContainer> From<NMFeagiAdvancedQuanted<C>> for NMModeled<C> {
    fn from(value: NMFeagiAdvancedQuanted<C>) -> Self {
        NMModeled::FeagiAdvanced(value)
    }
}

impl<C: NMContainer> TryFrom<NMModeled<C>> for NMFeagiAdvancedQuanted<C> {
    type Error = NMAccessError;

    fn try_from(value: NMModeled<C>) -> Result<Self, Self::Error> {
        let actual = value.kind();
        match value {
            NMModeled::FeagiAdvanced(quanted) => Ok(quanted),
            _ => Err(NMAccessError::IncorrectModel { expected: "FeagiAdvanced", actual }),
        }
    }
}

//endregion
