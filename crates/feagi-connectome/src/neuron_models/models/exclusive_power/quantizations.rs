use crate::neuron_models::models::exclusive_power::{NMExclusivePowerDataCorticalInternal, NMExclusivePowerDataCorticalProperties, NMExclusivePowerDataCorticalScratch, NMExclusivePowerDataCorticalVariance, NMExclusivePowerDataNeuronInternal, NMExclusivePowerDataNeuronScratch, NMExclusivePowerDataNeuronVariance, NMExclusivePowerTrait};
use crate::neuron_models::models::kind::NMKind;
use crate::neuron_models::models::container::{NMAccessError, NMContainer, NMContainerMarker, NMMember, NMVisitor, NMVisitorMut, NMVisitorRef};
use crate::neuron_models::models::NMModeled;
use crate::neuron_models::neurogenesis_generator_uniform_defaults::{NeurogenesisGeneratorPrecursorUniformDefault, NeurogenesisGeneratorUniformDefault};
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::voxel::NeuronLayoutTagVoxel;
use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome_neural::neural_basis::quantization_levels::{FeagiIndexQuantization, MembranePotentialQuantization};
use half::{bf16, f16};
//region enums

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
/// Describes the quantization of the Exclusive Power Neuron Model
pub enum NMExclusivePowerQuantKind {
    Float16,
    BFloat16,
    Float32,
    Float64,
}

/// The data of container `C` for the ExclusivePower Neuron Model, with a variant per quantization
#[derive(Clone, Debug, ::serde::Serialize, ::serde::Deserialize)]
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

    /// The model and quantization of the data held
    pub const fn kind(&self) -> NMKind {
        NMKind::ExclusivePower(self.quant_kind())
    }

    /// Runs the visitor on the data held, with the member it is held for
    pub fn dispatch<V: NMVisitor<C>>(self, visitor: V) -> V::Output {
        match self {
            Self::Float16(data) => visitor.visit::<NMExclusivePowerQuantFloat16>(data),
            Self::BFloat16(data) => visitor.visit::<NMExclusivePowerQuantBFloat16>(data),
            Self::Float32(data) => visitor.visit::<NMExclusivePowerQuantFloat32>(data),
            Self::Float64(data) => visitor.visit::<NMExclusivePowerQuantFloat64>(data),
        }
    }

    /// Runs the visitor on a reference to the data held, with the member it is held for
    pub fn dispatch_ref<'a, V: NMVisitorRef<'a, C>>(&'a self, visitor: V) -> V::Output {
        match self {
            Self::Float16(data) => visitor.visit::<NMExclusivePowerQuantFloat16>(data),
            Self::BFloat16(data) => visitor.visit::<NMExclusivePowerQuantBFloat16>(data),
            Self::Float32(data) => visitor.visit::<NMExclusivePowerQuantFloat32>(data),
            Self::Float64(data) => visitor.visit::<NMExclusivePowerQuantFloat64>(data),
        }
    }

    /// Runs the visitor on a mutable reference to the data held, with the member it is held for
    pub fn dispatch_mut<'a, V: NMVisitorMut<'a, C>>(&'a mut self, visitor: V) -> V::Output {
        match self {
            Self::Float16(data) => visitor.visit::<NMExclusivePowerQuantFloat16>(data),
            Self::BFloat16(data) => visitor.visit::<NMExclusivePowerQuantBFloat16>(data),
            Self::Float32(data) => visitor.visit::<NMExclusivePowerQuantFloat32>(data),
            Self::Float64(data) => visitor.visit::<NMExclusivePowerQuantFloat64>(data),
        }
    }
}

/// The marker struct of a quantization of the ExclusivePower Neuron Model
pub type NMExclusivePowerQuantedKind = NMExclusivePowerQuanted<NMContainerMarker>;

impl NMExclusivePowerQuantedKind {
    /// The marker struct of quantization `NM` of this model
    pub fn from_nm_quant<NM: NMExclusivePowerTrait>() -> Self {
        match NM::QUANT_KIND {
            NMExclusivePowerQuantKind::Float16 => Self::Float16(NMExclusivePowerQuantFloat16),
            NMExclusivePowerQuantKind::BFloat16 => Self::BFloat16(NMExclusivePowerQuantBFloat16),
            NMExclusivePowerQuantKind::Float32 => Self::Float32(NMExclusivePowerQuantFloat32),
            NMExclusivePowerQuantKind::Float64 => Self::Float64(NMExclusivePowerQuantFloat64),
        }
    }
}

// The marker structs are unit structs, so the kind is fully described by its quantization
impl Copy for NMExclusivePowerQuantedKind {}

impl PartialEq for NMExclusivePowerQuantedKind {
    fn eq(&self, other: &Self) -> bool {
        self.quant_kind() == other.quant_kind()
    }
}

impl Eq for NMExclusivePowerQuantedKind {}

//endregion

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMExclusivePowerQuantFloat16;

impl NMExclusivePowerTrait for NMExclusivePowerQuantFloat16 {
}

impl NeuronModel for NMExclusivePowerQuantFloat16 {
    type NeuronLayoutTag = NeuronLayoutTagVoxel;

    type NeurogenesisScratchVarianceGeneratorType<FIQ: FeagiIndexQuantization> =
        super::neurogenesis::ExclusivePowerNeurogenesisDefaults<FIQ, Self>;

    type NeurogenesisGenerator<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorUniformDefault<FIQ, Self>;
    type NeurogenesisGeneratorPrecursor<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorPrecursorUniformDefault<FIQ, Self>;

    type CorticalDataProperties = NMExclusivePowerDataCorticalProperties<Self>;
    type CorticalDataInternal = NMExclusivePowerDataCorticalInternal<Self>;
    type CorticalDataScratch = NMExclusivePowerDataCorticalScratch<Self>;
    type CorticalDataVariance = NMExclusivePowerDataCorticalVariance<Self>;
    type NeuronDataInternal = NMExclusivePowerDataNeuronInternal<Self>;
    type NeuronDataScratch = NMExclusivePowerDataNeuronScratch<Self>;
    type NeuronDataVariance = NMExclusivePowerDataNeuronVariance<Self>;

    type QuantKind = NMExclusivePowerQuantKind;
    type QuantedKind = NMExclusivePowerQuantedKind;
    const QUANT_KIND: Self::QuantKind = NMExclusivePowerQuantKind::Float16;
    const QUANTED_KIND: Self::QuantedKind = NMExclusivePowerQuantedKind::Float16(Self);
}

impl MembranePotentialQuantization for NMExclusivePowerQuantFloat16
{
    type MembranePotentialQuant = f16;
}

impl From<NMExclusivePowerQuantFloat16> for NMExclusivePowerQuantedKind {
    fn from(value: NMExclusivePowerQuantFloat16) -> Self {
        NMExclusivePowerQuantedKind::Float16(value)
    }
}

impl TryFrom<NMExclusivePowerQuantedKind> for NMExclusivePowerQuantFloat16 {
    type Error = NMAccessError;

    fn try_from(value: NMExclusivePowerQuantedKind) -> Result<Self, Self::Error> {
        match value {
            NMExclusivePowerQuanted::Float16(marker) => Ok(marker),
            #[allow(unreachable_patterns)]
            other => Err(NMAccessError::IncorrectMember { expected: <Self as NMMember>::KIND, actual: other.kind() }),
        }
    }
}

impl NMMember for NMExclusivePowerQuantFloat16 {
    const KIND: NMKind = NMKind::ExclusivePower(NMExclusivePowerQuantKind::Float16);
    const MARKER: Self = Self;

    fn into_modeled<C: NMContainer>(data: C::Of<Self>) -> NMModeled<C> {
        NMModeled::ExclusivePower(NMExclusivePowerQuanted::Float16(data))
    }

    fn try_from_modeled<C: NMContainer>(value: NMModeled<C>) -> Result<C::Of<Self>, NMAccessError> {
        let actual = value.kind();
        match value {
            NMModeled::ExclusivePower(NMExclusivePowerQuanted::Float16(data)) => Ok(data),
            _ => Err(NMAccessError::IncorrectMember { expected: Self::KIND, actual }),
        }
    }

    fn try_ref_modeled<C: NMContainer>(value: &NMModeled<C>) -> Result<&C::Of<Self>, NMAccessError> {
        match value {
            NMModeled::ExclusivePower(NMExclusivePowerQuanted::Float16(data)) => Ok(data),
            _ => Err(NMAccessError::IncorrectMember { expected: Self::KIND, actual: value.kind() }),
        }
    }

    fn try_mut_modeled<C: NMContainer>(value: &mut NMModeled<C>) -> Result<&mut C::Of<Self>, NMAccessError> {
        let actual = value.kind();
        match value {
            NMModeled::ExclusivePower(NMExclusivePowerQuanted::Float16(data)) => Ok(data),
            _ => Err(NMAccessError::IncorrectMember { expected: Self::KIND, actual }),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMExclusivePowerQuantBFloat16;

impl NMExclusivePowerTrait for NMExclusivePowerQuantBFloat16 {
}

impl NeuronModel for NMExclusivePowerQuantBFloat16 {
    type NeuronLayoutTag = NeuronLayoutTagVoxel;

    type NeurogenesisScratchVarianceGeneratorType<FIQ: FeagiIndexQuantization> =
        super::neurogenesis::ExclusivePowerNeurogenesisDefaults<FIQ, Self>;

    type NeurogenesisGenerator<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorUniformDefault<FIQ, Self>;
    type NeurogenesisGeneratorPrecursor<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorPrecursorUniformDefault<FIQ, Self>;

    type CorticalDataProperties = NMExclusivePowerDataCorticalProperties<Self>;
    type CorticalDataInternal = NMExclusivePowerDataCorticalInternal<Self>;
    type CorticalDataScratch = NMExclusivePowerDataCorticalScratch<Self>;
    type CorticalDataVariance = NMExclusivePowerDataCorticalVariance<Self>;
    type NeuronDataInternal = NMExclusivePowerDataNeuronInternal<Self>;
    type NeuronDataScratch = NMExclusivePowerDataNeuronScratch<Self>;
    type NeuronDataVariance = NMExclusivePowerDataNeuronVariance<Self>;

    type QuantKind = NMExclusivePowerQuantKind;
    type QuantedKind = NMExclusivePowerQuantedKind;
    const QUANT_KIND: Self::QuantKind = NMExclusivePowerQuantKind::BFloat16;
    const QUANTED_KIND: Self::QuantedKind = NMExclusivePowerQuantedKind::BFloat16(Self);
}

impl MembranePotentialQuantization for NMExclusivePowerQuantBFloat16
{
    type MembranePotentialQuant = bf16;
}

impl From<NMExclusivePowerQuantBFloat16> for NMExclusivePowerQuantedKind {
    fn from(value: NMExclusivePowerQuantBFloat16) -> Self {
        NMExclusivePowerQuantedKind::BFloat16(value)
    }
}

impl TryFrom<NMExclusivePowerQuantedKind> for NMExclusivePowerQuantBFloat16 {
    type Error = NMAccessError;

    fn try_from(value: NMExclusivePowerQuantedKind) -> Result<Self, Self::Error> {
        match value {
            NMExclusivePowerQuanted::BFloat16(marker) => Ok(marker),
            #[allow(unreachable_patterns)]
            other => Err(NMAccessError::IncorrectMember { expected: <Self as NMMember>::KIND, actual: other.kind() }),
        }
    }
}

impl NMMember for NMExclusivePowerQuantBFloat16 {
    const KIND: NMKind = NMKind::ExclusivePower(NMExclusivePowerQuantKind::BFloat16);
    const MARKER: Self = Self;

    fn into_modeled<C: NMContainer>(data: C::Of<Self>) -> NMModeled<C> {
        NMModeled::ExclusivePower(NMExclusivePowerQuanted::BFloat16(data))
    }

    fn try_from_modeled<C: NMContainer>(value: NMModeled<C>) -> Result<C::Of<Self>, NMAccessError> {
        let actual = value.kind();
        match value {
            NMModeled::ExclusivePower(NMExclusivePowerQuanted::BFloat16(data)) => Ok(data),
            _ => Err(NMAccessError::IncorrectMember { expected: Self::KIND, actual }),
        }
    }

    fn try_ref_modeled<C: NMContainer>(value: &NMModeled<C>) -> Result<&C::Of<Self>, NMAccessError> {
        match value {
            NMModeled::ExclusivePower(NMExclusivePowerQuanted::BFloat16(data)) => Ok(data),
            _ => Err(NMAccessError::IncorrectMember { expected: Self::KIND, actual: value.kind() }),
        }
    }

    fn try_mut_modeled<C: NMContainer>(value: &mut NMModeled<C>) -> Result<&mut C::Of<Self>, NMAccessError> {
        let actual = value.kind();
        match value {
            NMModeled::ExclusivePower(NMExclusivePowerQuanted::BFloat16(data)) => Ok(data),
            _ => Err(NMAccessError::IncorrectMember { expected: Self::KIND, actual }),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMExclusivePowerQuantFloat32;

impl NMExclusivePowerTrait for NMExclusivePowerQuantFloat32 {
}

impl NeuronModel for NMExclusivePowerQuantFloat32 {
    type NeuronLayoutTag = NeuronLayoutTagVoxel;

    type NeurogenesisScratchVarianceGeneratorType<FIQ: FeagiIndexQuantization> =
        super::neurogenesis::ExclusivePowerNeurogenesisDefaults<FIQ, Self>;

    type NeurogenesisGenerator<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorUniformDefault<FIQ, Self>;
    type NeurogenesisGeneratorPrecursor<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorPrecursorUniformDefault<FIQ, Self>;

    type CorticalDataProperties = NMExclusivePowerDataCorticalProperties<Self>;
    type CorticalDataInternal = NMExclusivePowerDataCorticalInternal<Self>;
    type CorticalDataScratch = NMExclusivePowerDataCorticalScratch<Self>;
    type CorticalDataVariance = NMExclusivePowerDataCorticalVariance<Self>;
    type NeuronDataInternal = NMExclusivePowerDataNeuronInternal<Self>;
    type NeuronDataScratch = NMExclusivePowerDataNeuronScratch<Self>;
    type NeuronDataVariance = NMExclusivePowerDataNeuronVariance<Self>;

    type QuantKind = NMExclusivePowerQuantKind;
    type QuantedKind = NMExclusivePowerQuantedKind;
    const QUANT_KIND: Self::QuantKind = NMExclusivePowerQuantKind::Float32;
    const QUANTED_KIND: Self::QuantedKind = NMExclusivePowerQuantedKind::Float32(Self);
}

impl MembranePotentialQuantization for NMExclusivePowerQuantFloat32
{
    type MembranePotentialQuant = f32;
}

impl From<NMExclusivePowerQuantFloat32> for NMExclusivePowerQuantedKind {
    fn from(value: NMExclusivePowerQuantFloat32) -> Self {
        NMExclusivePowerQuantedKind::Float32(value)
    }
}

impl TryFrom<NMExclusivePowerQuantedKind> for NMExclusivePowerQuantFloat32 {
    type Error = NMAccessError;

    fn try_from(value: NMExclusivePowerQuantedKind) -> Result<Self, Self::Error> {
        match value {
            NMExclusivePowerQuanted::Float32(marker) => Ok(marker),
            #[allow(unreachable_patterns)]
            other => Err(NMAccessError::IncorrectMember { expected: <Self as NMMember>::KIND, actual: other.kind() }),
        }
    }
}

impl NMMember for NMExclusivePowerQuantFloat32 {
    const KIND: NMKind = NMKind::ExclusivePower(NMExclusivePowerQuantKind::Float32);
    const MARKER: Self = Self;

    fn into_modeled<C: NMContainer>(data: C::Of<Self>) -> NMModeled<C> {
        NMModeled::ExclusivePower(NMExclusivePowerQuanted::Float32(data))
    }

    fn try_from_modeled<C: NMContainer>(value: NMModeled<C>) -> Result<C::Of<Self>, NMAccessError> {
        let actual = value.kind();
        match value {
            NMModeled::ExclusivePower(NMExclusivePowerQuanted::Float32(data)) => Ok(data),
            _ => Err(NMAccessError::IncorrectMember { expected: Self::KIND, actual }),
        }
    }

    fn try_ref_modeled<C: NMContainer>(value: &NMModeled<C>) -> Result<&C::Of<Self>, NMAccessError> {
        match value {
            NMModeled::ExclusivePower(NMExclusivePowerQuanted::Float32(data)) => Ok(data),
            _ => Err(NMAccessError::IncorrectMember { expected: Self::KIND, actual: value.kind() }),
        }
    }

    fn try_mut_modeled<C: NMContainer>(value: &mut NMModeled<C>) -> Result<&mut C::Of<Self>, NMAccessError> {
        let actual = value.kind();
        match value {
            NMModeled::ExclusivePower(NMExclusivePowerQuanted::Float32(data)) => Ok(data),
            _ => Err(NMAccessError::IncorrectMember { expected: Self::KIND, actual }),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMExclusivePowerQuantFloat64;

impl NMExclusivePowerTrait for NMExclusivePowerQuantFloat64 {
}

impl NeuronModel for NMExclusivePowerQuantFloat64 {
    type NeuronLayoutTag = NeuronLayoutTagVoxel;

    type NeurogenesisScratchVarianceGeneratorType<FIQ: FeagiIndexQuantization> =
        super::neurogenesis::ExclusivePowerNeurogenesisDefaults<FIQ, Self>;

    type NeurogenesisGenerator<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorUniformDefault<FIQ, Self>;
    type NeurogenesisGeneratorPrecursor<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorPrecursorUniformDefault<FIQ, Self>;

    type CorticalDataProperties = NMExclusivePowerDataCorticalProperties<Self>;
    type CorticalDataInternal = NMExclusivePowerDataCorticalInternal<Self>;
    type CorticalDataScratch = NMExclusivePowerDataCorticalScratch<Self>;
    type CorticalDataVariance = NMExclusivePowerDataCorticalVariance<Self>;
    type NeuronDataInternal = NMExclusivePowerDataNeuronInternal<Self>;
    type NeuronDataScratch = NMExclusivePowerDataNeuronScratch<Self>;
    type NeuronDataVariance = NMExclusivePowerDataNeuronVariance<Self>;

    type QuantKind = NMExclusivePowerQuantKind;
    type QuantedKind = NMExclusivePowerQuantedKind;
    const QUANT_KIND: Self::QuantKind = NMExclusivePowerQuantKind::Float64;
    const QUANTED_KIND: Self::QuantedKind = NMExclusivePowerQuantedKind::Float64(Self);
}

impl MembranePotentialQuantization for NMExclusivePowerQuantFloat64
{
    type MembranePotentialQuant = f64;
}

impl From<NMExclusivePowerQuantFloat64> for NMExclusivePowerQuantedKind {
    fn from(value: NMExclusivePowerQuantFloat64) -> Self {
        NMExclusivePowerQuantedKind::Float64(value)
    }
}

impl TryFrom<NMExclusivePowerQuantedKind> for NMExclusivePowerQuantFloat64 {
    type Error = NMAccessError;

    fn try_from(value: NMExclusivePowerQuantedKind) -> Result<Self, Self::Error> {
        match value {
            NMExclusivePowerQuanted::Float64(marker) => Ok(marker),
            #[allow(unreachable_patterns)]
            other => Err(NMAccessError::IncorrectMember { expected: <Self as NMMember>::KIND, actual: other.kind() }),
        }
    }
}

impl NMMember for NMExclusivePowerQuantFloat64 {
    const KIND: NMKind = NMKind::ExclusivePower(NMExclusivePowerQuantKind::Float64);
    const MARKER: Self = Self;

    fn into_modeled<C: NMContainer>(data: C::Of<Self>) -> NMModeled<C> {
        NMModeled::ExclusivePower(NMExclusivePowerQuanted::Float64(data))
    }

    fn try_from_modeled<C: NMContainer>(value: NMModeled<C>) -> Result<C::Of<Self>, NMAccessError> {
        let actual = value.kind();
        match value {
            NMModeled::ExclusivePower(NMExclusivePowerQuanted::Float64(data)) => Ok(data),
            _ => Err(NMAccessError::IncorrectMember { expected: Self::KIND, actual }),
        }
    }

    fn try_ref_modeled<C: NMContainer>(value: &NMModeled<C>) -> Result<&C::Of<Self>, NMAccessError> {
        match value {
            NMModeled::ExclusivePower(NMExclusivePowerQuanted::Float64(data)) => Ok(data),
            _ => Err(NMAccessError::IncorrectMember { expected: Self::KIND, actual: value.kind() }),
        }
    }

    fn try_mut_modeled<C: NMContainer>(value: &mut NMModeled<C>) -> Result<&mut C::Of<Self>, NMAccessError> {
        let actual = value.kind();
        match value {
            NMModeled::ExclusivePower(NMExclusivePowerQuanted::Float64(data)) => Ok(data),
            _ => Err(NMAccessError::IncorrectMember { expected: Self::KIND, actual }),
        }
    }
}

//region Model Level Conversions

impl From<NMExclusivePowerQuantKind> for NMKind {
    fn from(value: NMExclusivePowerQuantKind) -> Self {
        NMKind::ExclusivePower(value)
    }
}

impl TryFrom<NMKind> for NMExclusivePowerQuantKind {
    type Error = NMAccessError;

    fn try_from(value: NMKind) -> Result<Self, Self::Error> {
        match value {
            NMKind::ExclusivePower(quant) => Ok(quant),
            actual => Err(NMAccessError::IncorrectModel { expected: "ExclusivePower", actual }),
        }
    }
}

impl<C: NMContainer> From<NMExclusivePowerQuanted<C>> for NMModeled<C> {
    fn from(value: NMExclusivePowerQuanted<C>) -> Self {
        NMModeled::ExclusivePower(value)
    }
}

impl<C: NMContainer> TryFrom<NMModeled<C>> for NMExclusivePowerQuanted<C> {
    type Error = NMAccessError;

    fn try_from(value: NMModeled<C>) -> Result<Self, Self::Error> {
        let actual = value.kind();
        match value {
            NMModeled::ExclusivePower(quanted) => Ok(quanted),
            _ => Err(NMAccessError::IncorrectModel { expected: "ExclusivePower", actual }),
        }
    }
}

//endregion
