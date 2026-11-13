use crate::neuron_models::models::memory_fire_hashing::{NMMemoryFireHashingDataCorticalInternal, NMMemoryFireHashingDataCorticalProperties, NMMemoryFireHashingDataCorticalScratch, NMMemoryFireHashingDataCorticalVariance, NMMemoryFireHashingDataNeuronInternal, NMMemoryFireHashingDataNeuronScratch, NMMemoryFireHashingDataNeuronVariance, NMMemoryFireHashingTrait};
use crate::neuron_models::models::kind::NMKind;
use crate::neuron_models::models::container::{NMAccessError, NMContainer, NMContainerMarker, NMMember, NMVisitor, NMVisitorMut, NMVisitorRef};
use crate::neuron_models::models::NMModeled;
use crate::neuron_models::neurogenesis_generator_uniform_defaults::{NeurogenesisGeneratorPrecursorUniformDefault, NeurogenesisGeneratorUniformDefault};
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::single_stepped_dynamic::NeuronLayoutTagSingleSteppedDynamic;
use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome_neural::neural_basis::quantization_levels::{FeagiIndexQuantization, MembranePotentialQuantization};
//region enums

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
/// Describes the quantization of the Memory Fire Hashing Neuron Model
pub enum NMMemoryFireHashingQuantKind {
    Standard32Bit,
}

/// The data of container `C` for the MemoryFireHashing Neuron Model, with a variant per quantization
#[derive(Clone, Debug, ::serde::Serialize, ::serde::Deserialize)]
#[serde(bound(
    serialize = "C::Of<NMMemoryFireHashingQuantStandard32Bit>: ::serde::Serialize",
    deserialize = "C::Of<NMMemoryFireHashingQuantStandard32Bit>: ::serde::Deserialize<'de>"
))]
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

    /// The model and quantization of the data held
    pub const fn kind(&self) -> NMKind {
        NMKind::MemoryFireHashing(self.quant_kind())
    }

    /// Runs the visitor on the data held, with the member it is held for
    pub fn dispatch<V: NMVisitor<C>>(self, visitor: V) -> V::Output {
        match self {
            Self::Standard32Bit(data) => visitor.visit::<NMMemoryFireHashingQuantStandard32Bit>(data),
        }
    }

    /// Runs the visitor on a reference to the data held, with the member it is held for
    pub fn dispatch_ref<'a, V: NMVisitorRef<'a, C>>(&'a self, visitor: V) -> V::Output {
        match self {
            Self::Standard32Bit(data) => visitor.visit::<NMMemoryFireHashingQuantStandard32Bit>(data),
        }
    }

    /// Runs the visitor on a mutable reference to the data held, with the member it is held for
    pub fn dispatch_mut<'a, V: NMVisitorMut<'a, C>>(&'a mut self, visitor: V) -> V::Output {
        match self {
            Self::Standard32Bit(data) => visitor.visit::<NMMemoryFireHashingQuantStandard32Bit>(data),
        }
    }
}

/// The marker struct of a quantization of the MemoryFireHashing Neuron Model
pub type NMMemoryFireHashingQuantedKind = NMMemoryFireHashingQuanted<NMContainerMarker>;

impl NMMemoryFireHashingQuantedKind {
    /// The marker struct of quantization `NM` of this model
    pub fn from_nm_quant<NM: NMMemoryFireHashingTrait>() -> Self {
        match NM::QUANT_KIND {
            NMMemoryFireHashingQuantKind::Standard32Bit => Self::Standard32Bit(NMMemoryFireHashingQuantStandard32Bit),
        }
    }
}

// The marker structs are unit structs, so the kind is fully described by its quantization
impl Copy for NMMemoryFireHashingQuantedKind {}

impl PartialEq for NMMemoryFireHashingQuantedKind {
    fn eq(&self, other: &Self) -> bool {
        self.quant_kind() == other.quant_kind()
    }
}

impl Eq for NMMemoryFireHashingQuantedKind {}

//endregion

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMMemoryFireHashingQuantStandard32Bit;

impl NMMemoryFireHashingTrait for NMMemoryFireHashingQuantStandard32Bit {
    type Period = f32;
}

impl NeuronModel for NMMemoryFireHashingQuantStandard32Bit {
    type NeuronLayoutTag = NeuronLayoutTagSingleSteppedDynamic;

    type NeurogenesisScratchVarianceGeneratorType<FIQ: FeagiIndexQuantization> =
        super::neurogenesis::MemoryFireHashingNeurogenesisDefaults<FIQ, Self>;

    type NeurogenesisGenerator<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorUniformDefault<FIQ, Self>;
    type NeurogenesisGeneratorPrecursor<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorPrecursorUniformDefault<FIQ, Self>;

    type CorticalDataProperties = NMMemoryFireHashingDataCorticalProperties<Self>;
    type CorticalDataInternal = NMMemoryFireHashingDataCorticalInternal<Self>;
    type CorticalDataScratch = NMMemoryFireHashingDataCorticalScratch<Self>;
    type CorticalDataVariance = NMMemoryFireHashingDataCorticalVariance<Self>;
    type NeuronDataInternal = NMMemoryFireHashingDataNeuronInternal<Self>;
    type NeuronDataScratch = NMMemoryFireHashingDataNeuronScratch<Self>;
    type NeuronDataVariance = NMMemoryFireHashingDataNeuronVariance<Self>;

    type QuantKind = NMMemoryFireHashingQuantKind;
    type QuantedKind = NMMemoryFireHashingQuantedKind;
    const QUANT_KIND: Self::QuantKind = NMMemoryFireHashingQuantKind::Standard32Bit;
    const QUANTED_KIND: Self::QuantedKind = NMMemoryFireHashingQuantedKind::Standard32Bit(Self);
}

impl MembranePotentialQuantization for NMMemoryFireHashingQuantStandard32Bit
{
    type MembranePotentialQuant = f32;
}

impl From<NMMemoryFireHashingQuantStandard32Bit> for NMMemoryFireHashingQuantedKind {
    fn from(value: NMMemoryFireHashingQuantStandard32Bit) -> Self {
        NMMemoryFireHashingQuantedKind::Standard32Bit(value)
    }
}

impl TryFrom<NMMemoryFireHashingQuantedKind> for NMMemoryFireHashingQuantStandard32Bit {
    type Error = NMAccessError;

    fn try_from(value: NMMemoryFireHashingQuantedKind) -> Result<Self, Self::Error> {
        match value {
            NMMemoryFireHashingQuanted::Standard32Bit(marker) => Ok(marker),
            #[allow(unreachable_patterns)]
            other => Err(NMAccessError::IncorrectMember { expected: <Self as NMMember>::KIND, actual: other.kind() }),
        }
    }
}

impl NMMember for NMMemoryFireHashingQuantStandard32Bit {
    const KIND: NMKind = NMKind::MemoryFireHashing(NMMemoryFireHashingQuantKind::Standard32Bit);
    const MARKER: Self = Self;

    fn into_modeled<C: NMContainer>(data: C::Of<Self>) -> NMModeled<C> {
        NMModeled::MemoryFireHashing(NMMemoryFireHashingQuanted::Standard32Bit(data))
    }

    fn try_from_modeled<C: NMContainer>(value: NMModeled<C>) -> Result<C::Of<Self>, NMAccessError> {
        let actual = value.kind();
        match value {
            NMModeled::MemoryFireHashing(NMMemoryFireHashingQuanted::Standard32Bit(data)) => Ok(data),
            _ => Err(NMAccessError::IncorrectMember { expected: Self::KIND, actual }),
        }
    }

    fn try_ref_modeled<C: NMContainer>(value: &NMModeled<C>) -> Result<&C::Of<Self>, NMAccessError> {
        match value {
            NMModeled::MemoryFireHashing(NMMemoryFireHashingQuanted::Standard32Bit(data)) => Ok(data),
            _ => Err(NMAccessError::IncorrectMember { expected: Self::KIND, actual: value.kind() }),
        }
    }

    fn try_mut_modeled<C: NMContainer>(value: &mut NMModeled<C>) -> Result<&mut C::Of<Self>, NMAccessError> {
        let actual = value.kind();
        match value {
            NMModeled::MemoryFireHashing(NMMemoryFireHashingQuanted::Standard32Bit(data)) => Ok(data),
            _ => Err(NMAccessError::IncorrectMember { expected: Self::KIND, actual }),
        }
    }
}

//region Model Level Conversions

impl From<NMMemoryFireHashingQuantKind> for NMKind {
    fn from(value: NMMemoryFireHashingQuantKind) -> Self {
        NMKind::MemoryFireHashing(value)
    }
}

impl TryFrom<NMKind> for NMMemoryFireHashingQuantKind {
    type Error = NMAccessError;

    fn try_from(value: NMKind) -> Result<Self, Self::Error> {
        match value {
            NMKind::MemoryFireHashing(quant) => Ok(quant),
            actual => Err(NMAccessError::IncorrectModel { expected: "MemoryFireHashing", actual }),
        }
    }
}

impl<C: NMContainer> From<NMMemoryFireHashingQuanted<C>> for NMModeled<C> {
    fn from(value: NMMemoryFireHashingQuanted<C>) -> Self {
        NMModeled::MemoryFireHashing(value)
    }
}

impl<C: NMContainer> TryFrom<NMModeled<C>> for NMMemoryFireHashingQuanted<C> {
    type Error = NMAccessError;

    fn try_from(value: NMModeled<C>) -> Result<Self, Self::Error> {
        let actual = value.kind();
        match value {
            NMModeled::MemoryFireHashing(quanted) => Ok(quanted),
            _ => Err(NMAccessError::IncorrectModel { expected: "MemoryFireHashing", actual }),
        }
    }
}

//endregion
