use feagi_basis::neurons::CorticalAreaNeuronLocalIndex;
use crate::neuron_model::model_data::NeuronModelUniversalCorticalProperties;
use crate::neuron_model::neurogenesis::by_generator::NeurogenesisByGenerator;
use crate::neuron_model::neurogenesis::from_storage::NeurogenesisFromStorage;
use crate::neuron_model::neurogenesis::NeurogenesisMethod;
use crate::neuron_model::neuron_layout::NeuronLayoutTag;
use crate::neuron_model::NeuronModel;
use crate::quantization_levels::FeagiIndexQuantization;

#[derive(Clone, Debug)]
pub enum NeurogenesisFromStorageOrByGenerator<FIQ, NM, NBG>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
    NBG: NeurogenesisByGenerator<FIQ, NM>
{
    FromStorage(NeurogenesisFromStorage<FIQ, NM>),
    ByGenerator(NBG)
}

/// How a cortical area of model `NM` gets spawned: either from stored neuron data, or by that
/// model's own generator. The [`NeurogenesisFromStorageOrByGenerator`] of `NM`, its generator fixed
/// to `NM::NeurogenesisGenerator`, as `NeuronLayoutOf` is the layout of a model
pub type NeurogenesisFromStorageOrByGeneratorOf<FIQ, NM> =
    NeurogenesisFromStorageOrByGenerator<FIQ, NM, <NM as NeuronModel>::NeurogenesisByGeneratorType<FIQ>>;

impl<FIQ, NM, NBG> NeurogenesisMethod<FIQ, NM> for NeurogenesisFromStorageOrByGenerator<FIQ, NM, NBG>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
    NBG: NeurogenesisByGenerator<FIQ, NM>
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