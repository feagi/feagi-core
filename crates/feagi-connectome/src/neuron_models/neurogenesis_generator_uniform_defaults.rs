//! A neurogenesis generator starting every neuron of a cortical area from the same values. Any
//! neuron model may use it as its `NeuronModel::NeurogenesisGenerator`. Users of a model, such as
//! cortical types, wrap it to fill in their own defaults

use feagi_basis::neurons::CorticalAreaNeuronLocalIndex;
use feagi_connectome_neural::neural_basis::neuron_model::model_data::NeuronModelUniversalCorticalProperties;
use feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::by_generator::NeurogenesisGenerator;
use feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::from_storage::NeurogenesisStorage;
use feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::{NeurogenesisMethod, NeurogenesisScratchVarianceGenerator};
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::{CorticalLayoutContextOf, NeuronLayoutOf};
use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
use crate::feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::NeuronLayoutTag;

/// Probably the most common shared cortical area generator.
/// Generates a cortical area from fixed starting values: every neuron starts with the same
/// internal data. Its scratch and variance data come from the model's
/// `NeuronModel::NeurogenesisDefaults`. This allows different cortical models to specify their own
/// default cortical 
#[derive(Clone, Debug, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
// Every field is serializable through the traits of FIQ and NM, not the bounds serde infers
#[serde(bound = "")]
pub struct NeurogenesisGeneratorUniformDefault<FIQ: FeagiIndexQuantization, NM: NeuronModel> {
    /// How the neurons of the area are laid out
    pub layout: NeuronLayoutOf<NM, FIQ>,
    /// Cortical level data used by all models
    pub universal_properties: NeuronModelUniversalCorticalProperties<NM>,
    pub cortical_properties: NM::CorticalDataProperties,
    pub cortical_internal: NM::CorticalDataInternal,
    pub cortical_variance: NM::CorticalDataVariance,
    /// The internal data every neuron of the area starts with
    pub neuron_internal: NM::NeuronDataInternal,
}



impl<FIQ: FeagiIndexQuantization, NM: NeuronModel> NeurogenesisMethod<FIQ, NM> for NeurogenesisGeneratorUniformDefault<FIQ, NM> {
    fn get_layout(&self) -> &NeuronLayoutOf<NM, FIQ> {
        &self.layout
    }

    fn get_universal_cortical_properties(&self) -> NeuronModelUniversalCorticalProperties<NM> {
        self.universal_properties
    }

    fn into_model_cortical_data(self) -> (NM::CorticalDataProperties, NM::CorticalDataInternal) {
        (self.cortical_properties, self.cortical_internal)
    }

    fn clone_model_cortical_data(&self) -> (NM::CorticalDataProperties, NM::CorticalDataInternal) {
        (self.cortical_properties.clone(), self.cortical_internal.clone())
    }

    fn get_model_cortical_variance(&self) -> &NM::CorticalDataVariance {
        &self.cortical_variance
    }

    fn spawn_neuron_data_at(&self, index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>) -> (NM::NeuronDataInternal, NM::NeuronDataScratch) {
        let internal = self.neuron_internal.clone();
        let scratch = NM::generate_neuron_scratch(index, &self.layout, &internal);
        (internal, scratch)
    }
}

impl<FIQ: FeagiIndexQuantization, NM: NeuronModel> NeurogenesisGenerator<FIQ, NM> for NeurogenesisGeneratorUniformDefault<FIQ, NM> {

    type GeneratorInitParameters = NeurogenesisGeneratorUniformDefaultInitParameters<NM>;

    /// The internal data every neuron starts with, which storage does not hold as a single value
    type OtherDataNeededForFromStorage = NM::NeuronDataInternal;

    fn new(input: Self::GeneratorInitParameters, layout: NeuronLayoutOf<NM, FIQ>) -> Self {
        // NOTE: Layout Restrictor checks should be left to the Cortical Type
        Self {
            layout,
            universal_properties: input.universal_properties,
            cortical_properties: input.cortical_properties,
            cortical_internal: input.cortical_internal,
            cortical_variance: input.cortical_variance,
            neuron_internal: input.neuron_internal,
        }
    }

    /// Keeps the cortical data of the stored area, with every neuron starting from `neuron_internal`
    /// rather than its stored data
    fn new_generator_from_storage(storage: &NeurogenesisStorage<FIQ, NM>, default_uniform_neuron_internal_data: NM::NeuronDataInternal) -> Self {
        let (cortical_properties, cortical_internal) = storage.clone_model_cortical_data();
        Self {
            layout: *storage.get_layout(),
            universal_properties: storage.get_universal_cortical_properties(),
            cortical_properties,
            cortical_internal,
            cortical_variance: storage.get_model_cortical_variance().clone(),
            neuron_internal: default_uniform_neuron_internal_data,
        }
    }
}

/// The data of a `NeurogenesisGeneratorUniformDefault` generator
pub struct NeurogenesisGeneratorUniformDefaultInitParameters<NM: NeuronModel> {
    /// Cortical level data used by all models
    pub universal_properties: NeuronModelUniversalCorticalProperties<NM>,
    pub cortical_properties: NM::CorticalDataProperties,
    pub cortical_internal: NM::CorticalDataInternal,
    pub cortical_variance: NM::CorticalDataVariance,
    /// The internal data every neuron of the area starts with
    pub neuron_internal: NM::NeuronDataInternal,
}
