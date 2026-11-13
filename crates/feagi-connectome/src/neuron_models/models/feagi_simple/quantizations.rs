use crate::neuron_models::models::feagi_simple::{NMFeagiSimpleDataCorticalInternal, NMFeagiSimpleDataCorticalProperties, NMFeagiSimpleDataCorticalScratch, NMFeagiSimpleDataCorticalVariance, NMFeagiSimpleDataNeuronInternal, NMFeagiSimpleDataNeuronScratch, NMFeagiSimpleDataNeuronVariance, NMFeagiSimpleTrait};
use crate::neuron_models::models::container::{NMAccessError, NMContainer, NMContainerMarker, NMMember, NMVisitor, NMVisitorMut, NMVisitorRef};
use crate::neuron_models::models::NMModeled;
use crate::neuron_models::neurogenesis_generator_uniform_defaults::{NeurogenesisGeneratorPrecursorUniformDefault, NeurogenesisGeneratorUniformDefault};
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::voxel::NeuronLayoutTagVoxel;
use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome_neural::neural_basis::quantization_levels::{FeagiIndexQuantization, MembranePotentialQuantization};
use crate::feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::from_storage::NeurogenesisStorage;
use crate::neuron_models::models::kind::NMKind;

//region enums

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
/// Describes the quantization of the Feagi Simple Neuron Model
pub enum NMFeagiSimpleQuantKind {
    Standard32Bit,
}

impl NMFeagiSimpleQuantKind {
    pub fn from_nm_quant<NM: NMFeagiSimpleTrait>() -> NMFeagiSimpleQuantKind {NM::QUANT_KIND}
}

/// The data of container `C` for the FeagiSimple Neuron Model, with a variant per quantization
#[derive(Clone, Debug, ::serde::Serialize, ::serde::Deserialize)]
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

    /// The model and quantization of the data held
    pub const fn kind(&self) -> NMKind {
        NMKind::FeagiSimple(self.quant_kind())
    }

    /// Runs the visitor on the data held, with the member it is held for
    pub fn dispatch<V: NMVisitor<C>>(self, visitor: V) -> V::Output {
        match self {
            Self::Standard32Bit(data) => visitor.visit::<NMFeagiSimpleQuantStandard32Bit>(data),
        }
    }

    /// Runs the visitor on a reference to the data held, with the member it is held for
    pub fn dispatch_ref<'a, V: NMVisitorRef<'a, C>>(&'a self, visitor: V) -> V::Output {
        match self {
            Self::Standard32Bit(data) => visitor.visit::<NMFeagiSimpleQuantStandard32Bit>(data),
        }
    }

    /// Runs the visitor on a mutable reference to the data held, with the member it is held for
    pub fn dispatch_mut<'a, V: NMVisitorMut<'a, C>>(&'a mut self, visitor: V) -> V::Output {
        match self {
            Self::Standard32Bit(data) => visitor.visit::<NMFeagiSimpleQuantStandard32Bit>(data),
        }
    }
}

/// The marker struct of a quantization of the FeagiSimple Neuron Model
pub type NMFeagiSimpleQuantedKind = NMFeagiSimpleQuanted<NMContainerMarker>;

impl NMFeagiSimpleQuantedKind {
    /// The marker struct of quantization `NM` of this model
    pub fn from_nm_quant<NM: NMFeagiSimpleTrait>() -> Self {
        match NM::QUANT_KIND {
            NMFeagiSimpleQuantKind::Standard32Bit => Self::Standard32Bit(NMFeagiSimpleQuantStandard32Bit),
        }
    }
}

// The marker structs are unit structs, so the kind is fully described by its quantization
impl Copy for NMFeagiSimpleQuantedKind {}

impl PartialEq for NMFeagiSimpleQuantedKind {
    fn eq(&self, other: &Self) -> bool {
        self.quant_kind() == other.quant_kind()
    }
}

impl Eq for NMFeagiSimpleQuantedKind {}


//endregion

//region Quant Levels

//region Standard32Bit

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMFeagiSimpleQuantStandard32Bit;

impl NMFeagiSimpleTrait for NMFeagiSimpleQuantStandard32Bit {
    type Period = f32;
}

impl NeuronModel for NMFeagiSimpleQuantStandard32Bit {
    type NeuronLayoutTag = NeuronLayoutTagVoxel;

    type NeurogenesisScratchVarianceGeneratorType<FIQ: FeagiIndexQuantization> =
        super::neurogenesis::NMFeagiSimpleScratchVarianceGenerator<FIQ, Self>;

    type NeurogenesisGenerator<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorUniformDefault<FIQ, Self>;
    type NeurogenesisGeneratorPrecursor<FIQ: FeagiIndexQuantization> = 
        NeurogenesisGeneratorPrecursorUniformDefault<FIQ, Self>;

    type CorticalDataProperties = NMFeagiSimpleDataCorticalProperties<Self>;
    type CorticalDataInternal = NMFeagiSimpleDataCorticalInternal<Self>;
    type CorticalDataScratch = NMFeagiSimpleDataCorticalScratch<Self>;
    type CorticalDataVariance = NMFeagiSimpleDataCorticalVariance<Self>;
    type NeuronDataInternal = NMFeagiSimpleDataNeuronInternal<Self>;
    type NeuronDataScratch = NMFeagiSimpleDataNeuronScratch<Self>;
    type NeuronDataVariance = NMFeagiSimpleDataNeuronVariance<Self>;

    type QuantKind = NMFeagiSimpleQuantKind;
    type QuantedKind = NMFeagiSimpleQuantedKind;

    const QUANT_KIND: Self::QuantKind = NMFeagiSimpleQuantKind::Standard32Bit;
    const QUANTED_KIND: Self::QuantedKind = NMFeagiSimpleQuantedKind::Standard32Bit(Self);
}

impl MembranePotentialQuantization for NMFeagiSimpleQuantStandard32Bit {
    type MembranePotentialQuant = f32;
}

impl From<NMFeagiSimpleQuantStandard32Bit> for NMFeagiSimpleQuantedKind {
    fn from(value: NMFeagiSimpleQuantStandard32Bit) -> Self {
        NMFeagiSimpleQuantedKind::Standard32Bit(value)
    }
}

impl TryFrom<NMFeagiSimpleQuantedKind> for NMFeagiSimpleQuantStandard32Bit {
    type Error = NMAccessError;

    fn try_from(value: NMFeagiSimpleQuantedKind) -> Result<Self, Self::Error> {
        match value {
            NMFeagiSimpleQuanted::Standard32Bit(marker) => Ok(marker),
            #[allow(unreachable_patterns)]
            other => Err(NMAccessError::IncorrectMember { expected: <Self as NMMember>::KIND, actual: other.kind() }),
        }
    }
}

impl NMMember for NMFeagiSimpleQuantStandard32Bit {
    const KIND: NMKind = NMKind::FeagiSimple(NMFeagiSimpleQuantKind::Standard32Bit);
    const MARKER: Self = Self;

    fn into_modeled<C: NMContainer>(data: C::Of<Self>) -> NMModeled<C> {
        NMModeled::FeagiSimple(NMFeagiSimpleQuanted::Standard32Bit(data))
    }

    fn try_from_modeled<C: NMContainer>(value: NMModeled<C>) -> Result<C::Of<Self>, NMAccessError> {
        let actual = value.kind();
        match value {
            NMModeled::FeagiSimple(NMFeagiSimpleQuanted::Standard32Bit(data)) => Ok(data),
            _ => Err(NMAccessError::IncorrectMember { expected: Self::KIND, actual }),
        }
    }

    fn try_ref_modeled<C: NMContainer>(value: &NMModeled<C>) -> Result<&C::Of<Self>, NMAccessError> {
        match value {
            NMModeled::FeagiSimple(NMFeagiSimpleQuanted::Standard32Bit(data)) => Ok(data),
            _ => Err(NMAccessError::IncorrectMember { expected: Self::KIND, actual: value.kind() }),
        }
    }

    fn try_mut_modeled<C: NMContainer>(value: &mut NMModeled<C>) -> Result<&mut C::Of<Self>, NMAccessError> {
        let actual = value.kind();
        match value {
            NMModeled::FeagiSimple(NMFeagiSimpleQuanted::Standard32Bit(data)) => Ok(data),
            _ => Err(NMAccessError::IncorrectMember { expected: Self::KIND, actual }),
        }
    }
}

//endregion

//endregion

//region Model Level Conversions

impl From<NMFeagiSimpleQuantKind> for NMKind {
    fn from(value: NMFeagiSimpleQuantKind) -> Self {
        NMKind::FeagiSimple(value)
    }
}

impl TryFrom<NMKind> for NMFeagiSimpleQuantKind {
    type Error = NMAccessError;

    fn try_from(value: NMKind) -> Result<Self, Self::Error> {
        match value {
            NMKind::FeagiSimple(quant) => Ok(quant),
            actual => Err(NMAccessError::IncorrectModel { expected: "FeagiSimple", actual }),
        }
    }
}

impl<C: NMContainer> From<NMFeagiSimpleQuanted<C>> for NMModeled<C> {
    fn from(value: NMFeagiSimpleQuanted<C>) -> Self {
        NMModeled::FeagiSimple(value)
    }
}

impl<C: NMContainer> TryFrom<NMModeled<C>> for NMFeagiSimpleQuanted<C> {
    type Error = NMAccessError;

    fn try_from(value: NMModeled<C>) -> Result<Self, Self::Error> {
        let actual = value.kind();
        match value {
            NMModeled::FeagiSimple(quanted) => Ok(quanted),
            _ => Err(NMAccessError::IncorrectModel { expected: "FeagiSimple", actual }),
        }
    }
}

//endregion
