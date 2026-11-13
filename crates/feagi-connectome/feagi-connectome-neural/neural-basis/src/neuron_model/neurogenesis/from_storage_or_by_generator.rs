use feagi_basis::neurons::CorticalAreaNeuronLocalIndex;
use crate::neuron_model::model_data::NeuronModelUniversalCorticalProperties;
use crate::neuron_model::neurogenesis::by_generator::{NeurogenesisGenerator, NeurogenesisGeneratorPrecursor};
use crate::neuron_model::neurogenesis::from_storage::NeurogenesisStorage;
use crate::neuron_model::neurogenesis::NeurogenesisMethod;
use crate::neuron_model::neuron_layout::{NeuronLayoutError, NeuronLayoutOf, NeuronLayoutRestrictor, NeuronLayoutTag};
use crate::neuron_model::NeuronModel;
use crate::quantization_levels::FeagiIndexQuantization;

#[derive(Clone, Debug, ::serde::Serialize, ::serde::Deserialize)]
// Storage is serializable through the traits of FIQ and NM, only the generator needs a bound
#[serde(bound(serialize = "NBG: ::serde::Serialize", deserialize = "NBG: ::serde::Deserialize<'de>"))]
pub enum NeurogenesisFromStorageOrByGenerator<FIQ, NM, NBG>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
    NBG: NeurogenesisGenerator<FIQ, NM>
{
    FromStorage(NeurogenesisStorage<FIQ, NM>),
    ByGenerator(NBG)
}

/// How a cortical area of model `NM` gets spawned: either from stored neuron data, or by that
/// model's own generator. The [`NeurogenesisFromStorageOrByGenerator`] of `NM`, its generator fixed
/// to `NM::NeurogenesisGenerator`, as `NeuronLayoutOf` is the layout of a model
pub type NeurogenesisFromStorageOrByGeneratorOf<FIQ, NM> =
    NeurogenesisFromStorageOrByGenerator<FIQ, NM, <NM as NeuronModel>::NeurogenesisGenerator<FIQ>>;

impl<FIQ, NM, NBG> NeurogenesisMethod<FIQ, NM> for NeurogenesisFromStorageOrByGenerator<FIQ, NM, NBG>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
    NBG: NeurogenesisGenerator<FIQ, NM>
{
    fn get_layout(&self) -> &<NM::NeuronLayoutTag as NeuronLayoutTag>::Layout<FIQ> {
        match &self {
            NeurogenesisFromStorageOrByGenerator::FromStorage(s) => {s.get_layout()}
            NeurogenesisFromStorageOrByGenerator::ByGenerator(g) => {g.get_layout()}
        }
    }

    fn get_universal_cortical_properties(&self) -> NeuronModelUniversalCorticalProperties<NM> {
        match &self {
            NeurogenesisFromStorageOrByGenerator::FromStorage(s) => {s.get_universal_cortical_properties()}
            NeurogenesisFromStorageOrByGenerator::ByGenerator(g) => {g.get_universal_cortical_properties()}
        }
    }

    fn into_model_cortical_data(self) -> (NM::CorticalDataProperties, NM::CorticalDataInternal) {
        match self {
            NeurogenesisFromStorageOrByGenerator::FromStorage(s) => {s.into_model_cortical_data()}
            NeurogenesisFromStorageOrByGenerator::ByGenerator(g) => {g.into_model_cortical_data()}
        }
    }

    fn clone_model_cortical_data(&self) -> (NM::CorticalDataProperties, NM::CorticalDataInternal) {
        match self {
            NeurogenesisFromStorageOrByGenerator::FromStorage(s) => {s.clone_model_cortical_data()}
            NeurogenesisFromStorageOrByGenerator::ByGenerator(g) => {g.clone_model_cortical_data()}
        }
    }

    fn get_model_cortical_variance(&self) -> &NM::CorticalDataVariance {
        match &self {
            NeurogenesisFromStorageOrByGenerator::FromStorage(s) => {s.get_model_cortical_variance()}
            NeurogenesisFromStorageOrByGenerator::ByGenerator(g) => {g.get_model_cortical_variance()}
        }
    }

    fn spawn_neuron_data_at(&self, index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>) -> (NM::NeuronDataInternal, NM::NeuronDataScratch) {
        match &self {
            NeurogenesisFromStorageOrByGenerator::FromStorage(s) => {s.spawn_neuron_data_at(index)}
            NeurogenesisFromStorageOrByGenerator::ByGenerator(g) => {g.spawn_neuron_data_at(index)}
        }
    }
}

/// The data needed to spawn a cortical area of model `NM`: either its stored neuron data, or the
/// precursor of that model's own generator. What a genome holds before the generator is built
#[derive(Clone, Debug, ::serde::Serialize, ::serde::Deserialize)]
// Both are serializable through the traits of FIQ and NM, not the bounds serde infers
#[serde(bound = "")]
pub enum NeurogenesisFromStorageOrByGeneratorPrecursor<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
{
    FromStorage(NeurogenesisStorage<FIQ, NM>),
    ByGeneratorPrecursor(NM::NeurogenesisGeneratorPrecursor<FIQ>),
}

impl<FIQ, NM> NeurogenesisFromStorageOrByGeneratorPrecursor<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
{
    /// Builds the generator from the precursor, if held. Storage is passed through as is
    pub fn into_storage_or_generator<Res: NeuronLayoutRestrictor<FIQ, NeuronLayout=NeuronLayoutOf<FIQ, NM>>>(
        self
    ) -> Result<NeurogenesisFromStorageOrByGeneratorOf<FIQ, NM>, NeuronLayoutError> {
        Ok(match self {
            Self::FromStorage(storage) => NeurogenesisFromStorageOrByGenerator::FromStorage(storage),
            Self::ByGeneratorPrecursor(precursor) => NeurogenesisFromStorageOrByGenerator::ByGenerator(precursor.into_generator::<Res>()?),
        })
    }
}
