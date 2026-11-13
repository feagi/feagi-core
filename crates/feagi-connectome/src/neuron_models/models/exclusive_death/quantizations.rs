use crate::neuron_models::models::exclusive_death::{NMExclusiveDeathDataCorticalInternal, NMExclusiveDeathDataCorticalProperties, NMExclusiveDeathDataCorticalScratch, NMExclusiveDeathDataCorticalVariance, NMExclusiveDeathDataNeuronInternal, NMExclusiveDeathDataNeuronScratch, NMExclusiveDeathDataNeuronVariance, NMExclusiveDeathTrait};
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
/// Describes the quantization of the Exclusive Death Neuron Model
pub enum NMExclusiveDeathQuantKind {
    Float16,
    BFloat16,
    Float32,
    Float64,
}

/// The data of container `C` for the ExclusiveDeath Neuron Model, with a variant per quantization
#[derive(Clone, Debug, ::serde::Serialize, ::serde::Deserialize)]
#[serde(bound(
    serialize = "C::Of<NMExclusiveDeathQuantFloat16>: ::serde::Serialize, C::Of<NMExclusiveDeathQuantBFloat16>: ::serde::Serialize, C::Of<NMExclusiveDeathQuantFloat32>: ::serde::Serialize, C::Of<NMExclusiveDeathQuantFloat64>: ::serde::Serialize",
    deserialize = "C::Of<NMExclusiveDeathQuantFloat16>: ::serde::Deserialize<'de>, C::Of<NMExclusiveDeathQuantBFloat16>: ::serde::Deserialize<'de>, C::Of<NMExclusiveDeathQuantFloat32>: ::serde::Deserialize<'de>, C::Of<NMExclusiveDeathQuantFloat64>: ::serde::Deserialize<'de>"
))]
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

    /// The model and quantization of the data held
    pub const fn kind(&self) -> NMKind {
        NMKind::ExclusiveDeath(self.quant_kind())
    }

    /// Runs the visitor on the data held, with the member it is held for
    pub fn dispatch<V: NMVisitor<C>>(self, visitor: V) -> V::Output {
        match self {
            Self::Float16(data) => visitor.visit::<NMExclusiveDeathQuantFloat16>(data),
            Self::BFloat16(data) => visitor.visit::<NMExclusiveDeathQuantBFloat16>(data),
            Self::Float32(data) => visitor.visit::<NMExclusiveDeathQuantFloat32>(data),
            Self::Float64(data) => visitor.visit::<NMExclusiveDeathQuantFloat64>(data),
        }
    }

    /// Runs the visitor on a reference to the data held, with the member it is held for
    pub fn dispatch_ref<'a, V: NMVisitorRef<'a, C>>(&'a self, visitor: V) -> V::Output {
        match self {
            Self::Float16(data) => visitor.visit::<NMExclusiveDeathQuantFloat16>(data),
            Self::BFloat16(data) => visitor.visit::<NMExclusiveDeathQuantBFloat16>(data),
            Self::Float32(data) => visitor.visit::<NMExclusiveDeathQuantFloat32>(data),
            Self::Float64(data) => visitor.visit::<NMExclusiveDeathQuantFloat64>(data),
        }
    }

    /// Runs the visitor on a mutable reference to the data held, with the member it is held for
    pub fn dispatch_mut<'a, V: NMVisitorMut<'a, C>>(&'a mut self, visitor: V) -> V::Output {
        match self {
            Self::Float16(data) => visitor.visit::<NMExclusiveDeathQuantFloat16>(data),
            Self::BFloat16(data) => visitor.visit::<NMExclusiveDeathQuantBFloat16>(data),
            Self::Float32(data) => visitor.visit::<NMExclusiveDeathQuantFloat32>(data),
            Self::Float64(data) => visitor.visit::<NMExclusiveDeathQuantFloat64>(data),
        }
    }
}

/// The marker struct of a quantization of the ExclusiveDeath Neuron Model
pub type NMExclusiveDeathQuantedKind = NMExclusiveDeathQuanted<NMContainerMarker>;

impl NMExclusiveDeathQuantedKind {
    /// The marker struct of quantization `NM` of this model
    pub fn from_nm_quant<NM: NMExclusiveDeathTrait>() -> Self {
        match NM::QUANT_KIND {
            NMExclusiveDeathQuantKind::Float16 => Self::Float16(NMExclusiveDeathQuantFloat16),
            NMExclusiveDeathQuantKind::BFloat16 => Self::BFloat16(NMExclusiveDeathQuantBFloat16),
            NMExclusiveDeathQuantKind::Float32 => Self::Float32(NMExclusiveDeathQuantFloat32),
            NMExclusiveDeathQuantKind::Float64 => Self::Float64(NMExclusiveDeathQuantFloat64),
        }
    }
}

// The marker structs are unit structs, so the kind is fully described by its quantization
impl Copy for NMExclusiveDeathQuantedKind {}

impl PartialEq for NMExclusiveDeathQuantedKind {
    fn eq(&self, other: &Self) -> bool {
        self.quant_kind() == other.quant_kind()
    }
}

impl Eq for NMExclusiveDeathQuantedKind {}

//endregion

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMExclusiveDeathQuantFloat16;

impl NMExclusiveDeathTrait for NMExclusiveDeathQuantFloat16 {
}

impl NeuronModel for NMExclusiveDeathQuantFloat16 {
    type NeuronLayoutTag = NeuronLayoutTagVoxel;

    type NeurogenesisScratchVarianceGeneratorType<FIQ: FeagiIndexQuantization> =
        super::neurogenesis::ExclusiveDeathNeurogenesisDefaults<FIQ, Self>;

    type NeurogenesisGenerator<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorUniformDefault<FIQ, Self>;
    type NeurogenesisGeneratorPrecursor<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorPrecursorUniformDefault<FIQ, Self>;

    type CorticalDataProperties = NMExclusiveDeathDataCorticalProperties<Self>;
    type CorticalDataInternal = NMExclusiveDeathDataCorticalInternal<Self>;
    type CorticalDataScratch = NMExclusiveDeathDataCorticalScratch<Self>;
    type CorticalDataVariance = NMExclusiveDeathDataCorticalVariance<Self>;
    type NeuronDataInternal = NMExclusiveDeathDataNeuronInternal<Self>;
    type NeuronDataScratch = NMExclusiveDeathDataNeuronScratch<Self>;
    type NeuronDataVariance = NMExclusiveDeathDataNeuronVariance<Self>;

    type QuantKind = NMExclusiveDeathQuantKind;
    type QuantedKind = NMExclusiveDeathQuantedKind;
    const QUANT_KIND: Self::QuantKind = NMExclusiveDeathQuantKind::Float16;
    const QUANTED_KIND: Self::QuantedKind = NMExclusiveDeathQuantedKind::Float16(Self);
}

impl MembranePotentialQuantization for NMExclusiveDeathQuantFloat16
{
    type MembranePotentialQuant = f16;
}

impl From<NMExclusiveDeathQuantFloat16> for NMExclusiveDeathQuantedKind {
    fn from(value: NMExclusiveDeathQuantFloat16) -> Self {
        NMExclusiveDeathQuantedKind::Float16(value)
    }
}

impl TryFrom<NMExclusiveDeathQuantedKind> for NMExclusiveDeathQuantFloat16 {
    type Error = NMAccessError;

    fn try_from(value: NMExclusiveDeathQuantedKind) -> Result<Self, Self::Error> {
        match value {
            NMExclusiveDeathQuanted::Float16(marker) => Ok(marker),
            #[allow(unreachable_patterns)]
            other => Err(NMAccessError::IncorrectMember { expected: <Self as NMMember>::KIND, actual: other.kind() }),
        }
    }
}

impl NMMember for NMExclusiveDeathQuantFloat16 {
    const KIND: NMKind = NMKind::ExclusiveDeath(NMExclusiveDeathQuantKind::Float16);
    const MARKER: Self = Self;

    fn into_modeled<C: NMContainer>(data: C::Of<Self>) -> NMModeled<C> {
        NMModeled::ExclusiveDeath(NMExclusiveDeathQuanted::Float16(data))
    }

    fn try_from_modeled<C: NMContainer>(value: NMModeled<C>) -> Result<C::Of<Self>, NMAccessError> {
        let actual = value.kind();
        match value {
            NMModeled::ExclusiveDeath(NMExclusiveDeathQuanted::Float16(data)) => Ok(data),
            _ => Err(NMAccessError::IncorrectMember { expected: Self::KIND, actual }),
        }
    }

    fn try_ref_modeled<C: NMContainer>(value: &NMModeled<C>) -> Result<&C::Of<Self>, NMAccessError> {
        match value {
            NMModeled::ExclusiveDeath(NMExclusiveDeathQuanted::Float16(data)) => Ok(data),
            _ => Err(NMAccessError::IncorrectMember { expected: Self::KIND, actual: value.kind() }),
        }
    }

    fn try_mut_modeled<C: NMContainer>(value: &mut NMModeled<C>) -> Result<&mut C::Of<Self>, NMAccessError> {
        let actual = value.kind();
        match value {
            NMModeled::ExclusiveDeath(NMExclusiveDeathQuanted::Float16(data)) => Ok(data),
            _ => Err(NMAccessError::IncorrectMember { expected: Self::KIND, actual }),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMExclusiveDeathQuantBFloat16;

impl NMExclusiveDeathTrait for NMExclusiveDeathQuantBFloat16 {
}

impl NeuronModel for NMExclusiveDeathQuantBFloat16 {
    type NeuronLayoutTag = NeuronLayoutTagVoxel;

    type NeurogenesisScratchVarianceGeneratorType<FIQ: FeagiIndexQuantization> =
        super::neurogenesis::ExclusiveDeathNeurogenesisDefaults<FIQ, Self>;

    type NeurogenesisGenerator<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorUniformDefault<FIQ, Self>;
    type NeurogenesisGeneratorPrecursor<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorPrecursorUniformDefault<FIQ, Self>;

    type CorticalDataProperties = NMExclusiveDeathDataCorticalProperties<Self>;
    type CorticalDataInternal = NMExclusiveDeathDataCorticalInternal<Self>;
    type CorticalDataScratch = NMExclusiveDeathDataCorticalScratch<Self>;
    type CorticalDataVariance = NMExclusiveDeathDataCorticalVariance<Self>;
    type NeuronDataInternal = NMExclusiveDeathDataNeuronInternal<Self>;
    type NeuronDataScratch = NMExclusiveDeathDataNeuronScratch<Self>;
    type NeuronDataVariance = NMExclusiveDeathDataNeuronVariance<Self>;

    type QuantKind = NMExclusiveDeathQuantKind;
    type QuantedKind = NMExclusiveDeathQuantedKind;
    const QUANT_KIND: Self::QuantKind = NMExclusiveDeathQuantKind::BFloat16;
    const QUANTED_KIND: Self::QuantedKind = NMExclusiveDeathQuantedKind::BFloat16(Self);
}

impl MembranePotentialQuantization for NMExclusiveDeathQuantBFloat16
{
    type MembranePotentialQuant = bf16;
}

impl From<NMExclusiveDeathQuantBFloat16> for NMExclusiveDeathQuantedKind {
    fn from(value: NMExclusiveDeathQuantBFloat16) -> Self {
        NMExclusiveDeathQuantedKind::BFloat16(value)
    }
}

impl TryFrom<NMExclusiveDeathQuantedKind> for NMExclusiveDeathQuantBFloat16 {
    type Error = NMAccessError;

    fn try_from(value: NMExclusiveDeathQuantedKind) -> Result<Self, Self::Error> {
        match value {
            NMExclusiveDeathQuanted::BFloat16(marker) => Ok(marker),
            #[allow(unreachable_patterns)]
            other => Err(NMAccessError::IncorrectMember { expected: <Self as NMMember>::KIND, actual: other.kind() }),
        }
    }
}

impl NMMember for NMExclusiveDeathQuantBFloat16 {
    const KIND: NMKind = NMKind::ExclusiveDeath(NMExclusiveDeathQuantKind::BFloat16);
    const MARKER: Self = Self;

    fn into_modeled<C: NMContainer>(data: C::Of<Self>) -> NMModeled<C> {
        NMModeled::ExclusiveDeath(NMExclusiveDeathQuanted::BFloat16(data))
    }

    fn try_from_modeled<C: NMContainer>(value: NMModeled<C>) -> Result<C::Of<Self>, NMAccessError> {
        let actual = value.kind();
        match value {
            NMModeled::ExclusiveDeath(NMExclusiveDeathQuanted::BFloat16(data)) => Ok(data),
            _ => Err(NMAccessError::IncorrectMember { expected: Self::KIND, actual }),
        }
    }

    fn try_ref_modeled<C: NMContainer>(value: &NMModeled<C>) -> Result<&C::Of<Self>, NMAccessError> {
        match value {
            NMModeled::ExclusiveDeath(NMExclusiveDeathQuanted::BFloat16(data)) => Ok(data),
            _ => Err(NMAccessError::IncorrectMember { expected: Self::KIND, actual: value.kind() }),
        }
    }

    fn try_mut_modeled<C: NMContainer>(value: &mut NMModeled<C>) -> Result<&mut C::Of<Self>, NMAccessError> {
        let actual = value.kind();
        match value {
            NMModeled::ExclusiveDeath(NMExclusiveDeathQuanted::BFloat16(data)) => Ok(data),
            _ => Err(NMAccessError::IncorrectMember { expected: Self::KIND, actual }),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMExclusiveDeathQuantFloat32;

impl NMExclusiveDeathTrait for NMExclusiveDeathQuantFloat32 {
}

impl NeuronModel for NMExclusiveDeathQuantFloat32 {
    type NeuronLayoutTag = NeuronLayoutTagVoxel;

    type NeurogenesisScratchVarianceGeneratorType<FIQ: FeagiIndexQuantization> =
        super::neurogenesis::ExclusiveDeathNeurogenesisDefaults<FIQ, Self>;

    type NeurogenesisGenerator<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorUniformDefault<FIQ, Self>;
    type NeurogenesisGeneratorPrecursor<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorPrecursorUniformDefault<FIQ, Self>;

    type CorticalDataProperties = NMExclusiveDeathDataCorticalProperties<Self>;
    type CorticalDataInternal = NMExclusiveDeathDataCorticalInternal<Self>;
    type CorticalDataScratch = NMExclusiveDeathDataCorticalScratch<Self>;
    type CorticalDataVariance = NMExclusiveDeathDataCorticalVariance<Self>;
    type NeuronDataInternal = NMExclusiveDeathDataNeuronInternal<Self>;
    type NeuronDataScratch = NMExclusiveDeathDataNeuronScratch<Self>;
    type NeuronDataVariance = NMExclusiveDeathDataNeuronVariance<Self>;

    type QuantKind = NMExclusiveDeathQuantKind;
    type QuantedKind = NMExclusiveDeathQuantedKind;
    const QUANT_KIND: Self::QuantKind = NMExclusiveDeathQuantKind::Float32;
    const QUANTED_KIND: Self::QuantedKind = NMExclusiveDeathQuantedKind::Float32(Self);
}

impl MembranePotentialQuantization for NMExclusiveDeathQuantFloat32
{
    type MembranePotentialQuant = f32;
}

impl From<NMExclusiveDeathQuantFloat32> for NMExclusiveDeathQuantedKind {
    fn from(value: NMExclusiveDeathQuantFloat32) -> Self {
        NMExclusiveDeathQuantedKind::Float32(value)
    }
}

impl TryFrom<NMExclusiveDeathQuantedKind> for NMExclusiveDeathQuantFloat32 {
    type Error = NMAccessError;

    fn try_from(value: NMExclusiveDeathQuantedKind) -> Result<Self, Self::Error> {
        match value {
            NMExclusiveDeathQuanted::Float32(marker) => Ok(marker),
            #[allow(unreachable_patterns)]
            other => Err(NMAccessError::IncorrectMember { expected: <Self as NMMember>::KIND, actual: other.kind() }),
        }
    }
}

impl NMMember for NMExclusiveDeathQuantFloat32 {
    const KIND: NMKind = NMKind::ExclusiveDeath(NMExclusiveDeathQuantKind::Float32);
    const MARKER: Self = Self;

    fn into_modeled<C: NMContainer>(data: C::Of<Self>) -> NMModeled<C> {
        NMModeled::ExclusiveDeath(NMExclusiveDeathQuanted::Float32(data))
    }

    fn try_from_modeled<C: NMContainer>(value: NMModeled<C>) -> Result<C::Of<Self>, NMAccessError> {
        let actual = value.kind();
        match value {
            NMModeled::ExclusiveDeath(NMExclusiveDeathQuanted::Float32(data)) => Ok(data),
            _ => Err(NMAccessError::IncorrectMember { expected: Self::KIND, actual }),
        }
    }

    fn try_ref_modeled<C: NMContainer>(value: &NMModeled<C>) -> Result<&C::Of<Self>, NMAccessError> {
        match value {
            NMModeled::ExclusiveDeath(NMExclusiveDeathQuanted::Float32(data)) => Ok(data),
            _ => Err(NMAccessError::IncorrectMember { expected: Self::KIND, actual: value.kind() }),
        }
    }

    fn try_mut_modeled<C: NMContainer>(value: &mut NMModeled<C>) -> Result<&mut C::Of<Self>, NMAccessError> {
        let actual = value.kind();
        match value {
            NMModeled::ExclusiveDeath(NMExclusiveDeathQuanted::Float32(data)) => Ok(data),
            _ => Err(NMAccessError::IncorrectMember { expected: Self::KIND, actual }),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMExclusiveDeathQuantFloat64;

impl NMExclusiveDeathTrait for NMExclusiveDeathQuantFloat64 {
}

impl NeuronModel for NMExclusiveDeathQuantFloat64 {
    type NeuronLayoutTag = NeuronLayoutTagVoxel;

    type NeurogenesisScratchVarianceGeneratorType<FIQ: FeagiIndexQuantization> =
        super::neurogenesis::ExclusiveDeathNeurogenesisDefaults<FIQ, Self>;

    type NeurogenesisGenerator<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorUniformDefault<FIQ, Self>;
    type NeurogenesisGeneratorPrecursor<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorPrecursorUniformDefault<FIQ, Self>;

    type CorticalDataProperties = NMExclusiveDeathDataCorticalProperties<Self>;
    type CorticalDataInternal = NMExclusiveDeathDataCorticalInternal<Self>;
    type CorticalDataScratch = NMExclusiveDeathDataCorticalScratch<Self>;
    type CorticalDataVariance = NMExclusiveDeathDataCorticalVariance<Self>;
    type NeuronDataInternal = NMExclusiveDeathDataNeuronInternal<Self>;
    type NeuronDataScratch = NMExclusiveDeathDataNeuronScratch<Self>;
    type NeuronDataVariance = NMExclusiveDeathDataNeuronVariance<Self>;

    type QuantKind = NMExclusiveDeathQuantKind;
    type QuantedKind = NMExclusiveDeathQuantedKind;
    const QUANT_KIND: Self::QuantKind = NMExclusiveDeathQuantKind::Float64;
    const QUANTED_KIND: Self::QuantedKind = NMExclusiveDeathQuantedKind::Float64(Self);
}

impl MembranePotentialQuantization for NMExclusiveDeathQuantFloat64
{
    type MembranePotentialQuant = f64;
}

impl From<NMExclusiveDeathQuantFloat64> for NMExclusiveDeathQuantedKind {
    fn from(value: NMExclusiveDeathQuantFloat64) -> Self {
        NMExclusiveDeathQuantedKind::Float64(value)
    }
}

impl TryFrom<NMExclusiveDeathQuantedKind> for NMExclusiveDeathQuantFloat64 {
    type Error = NMAccessError;

    fn try_from(value: NMExclusiveDeathQuantedKind) -> Result<Self, Self::Error> {
        match value {
            NMExclusiveDeathQuanted::Float64(marker) => Ok(marker),
            #[allow(unreachable_patterns)]
            other => Err(NMAccessError::IncorrectMember { expected: <Self as NMMember>::KIND, actual: other.kind() }),
        }
    }
}

impl NMMember for NMExclusiveDeathQuantFloat64 {
    const KIND: NMKind = NMKind::ExclusiveDeath(NMExclusiveDeathQuantKind::Float64);
    const MARKER: Self = Self;

    fn into_modeled<C: NMContainer>(data: C::Of<Self>) -> NMModeled<C> {
        NMModeled::ExclusiveDeath(NMExclusiveDeathQuanted::Float64(data))
    }

    fn try_from_modeled<C: NMContainer>(value: NMModeled<C>) -> Result<C::Of<Self>, NMAccessError> {
        let actual = value.kind();
        match value {
            NMModeled::ExclusiveDeath(NMExclusiveDeathQuanted::Float64(data)) => Ok(data),
            _ => Err(NMAccessError::IncorrectMember { expected: Self::KIND, actual }),
        }
    }

    fn try_ref_modeled<C: NMContainer>(value: &NMModeled<C>) -> Result<&C::Of<Self>, NMAccessError> {
        match value {
            NMModeled::ExclusiveDeath(NMExclusiveDeathQuanted::Float64(data)) => Ok(data),
            _ => Err(NMAccessError::IncorrectMember { expected: Self::KIND, actual: value.kind() }),
        }
    }

    fn try_mut_modeled<C: NMContainer>(value: &mut NMModeled<C>) -> Result<&mut C::Of<Self>, NMAccessError> {
        let actual = value.kind();
        match value {
            NMModeled::ExclusiveDeath(NMExclusiveDeathQuanted::Float64(data)) => Ok(data),
            _ => Err(NMAccessError::IncorrectMember { expected: Self::KIND, actual }),
        }
    }
}

//region Model Level Conversions

impl From<NMExclusiveDeathQuantKind> for NMKind {
    fn from(value: NMExclusiveDeathQuantKind) -> Self {
        NMKind::ExclusiveDeath(value)
    }
}

impl TryFrom<NMKind> for NMExclusiveDeathQuantKind {
    type Error = NMAccessError;

    fn try_from(value: NMKind) -> Result<Self, Self::Error> {
        match value {
            NMKind::ExclusiveDeath(quant) => Ok(quant),
            actual => Err(NMAccessError::IncorrectModel { expected: "ExclusiveDeath", actual }),
        }
    }
}

impl<C: NMContainer> From<NMExclusiveDeathQuanted<C>> for NMModeled<C> {
    fn from(value: NMExclusiveDeathQuanted<C>) -> Self {
        NMModeled::ExclusiveDeath(value)
    }
}

impl<C: NMContainer> TryFrom<NMModeled<C>> for NMExclusiveDeathQuanted<C> {
    type Error = NMAccessError;

    fn try_from(value: NMModeled<C>) -> Result<Self, Self::Error> {
        let actual = value.kind();
        match value {
            NMModeled::ExclusiveDeath(quanted) => Ok(quanted),
            _ => Err(NMAccessError::IncorrectModel { expected: "ExclusiveDeath", actual }),
        }
    }
}

//endregion
