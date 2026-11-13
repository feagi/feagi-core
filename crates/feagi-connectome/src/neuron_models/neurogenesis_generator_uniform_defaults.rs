//! A neurogenesis generator starting every neuron of a cortical area from the same values. Any
//! neuron model may use it as its `NeuronModel::NeurogenesisByGenerator`. Cortical types fill in
//! their own defaults. This is a very common generator

use crate::feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::by_generator::NeurogenesisByGenerator;
use feagi_basis::neurons::CorticalAreaNeuronLocalIndex;
use feagi_connectome_neural::neural_basis::neuron_model::model_data::NeuronModelUniversalCorticalProperties;
use feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::NeurogenesisMethod;
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::{CorticalLayoutContextOf, NeuronLayoutOf};
use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;

/// Probably the most common shared cortical area generator. Every neuron starts with the same
/// internal data. Its scratch and variance data come from the model's
/// `NeuronModel::NeurogenesisScratchVarianceDefaults`
#[derive(Clone, Debug, ::serde::Serialize, ::serde::Deserialize)]
// Every field is serializable through the traits of FIQ and NM, not the bounds serde infers
#[serde(bound = "")]
pub struct NeurogenesisGeneratorUniformDefault<FIQ: FeagiIndexQuantization, NM: NeuronModel> {
    /// Cortical level data used by all models
    pub universal_properties: NeuronModelUniversalCorticalProperties<NM>,
    /// How the neurons of the area are to be laid out, not yet validated
    pub cortical_context: CorticalLayoutContextOf<FIQ, NM>,
    pub cortical_properties: NM::CorticalDataProperties,
    pub cortical_internal: NM::CorticalDataInternal,
    pub cortical_variance: NM::CorticalDataVariance,
    /// The internal data every neuron of the area starts with
    pub neuron_internal: NM::NeuronDataInternal,
}

impl<FIQ: FeagiIndexQuantization, NM: NeuronModel> NeurogenesisMethod<FIQ, NM> for NeurogenesisGeneratorUniformDefault<FIQ, NM> {
    fn get_cortical_layout_context(&self) -> &CorticalLayoutContextOf<FIQ, NM> {
        &self.cortical_context
    }

    fn get_universal_cortical_properties(&self) -> &NeuronModelUniversalCorticalProperties<NM> {
        &self.universal_properties
    }

    fn get_model_cortical_properties(&self) -> &NM::CorticalDataProperties {
        &self.cortical_properties
    }

    fn get_model_cortical_internal(&self) -> &NM::CorticalDataInternal {
        &self.cortical_internal
    }

    fn get_model_cortical_variance(&self) -> &NM::CorticalDataVariance {
        &self.cortical_variance
    }

    fn into_model_cortical_data(self) -> (NM::CorticalDataProperties, NM::CorticalDataInternal) {
        (self.cortical_properties, self.cortical_internal)
    }

    fn spawn_neuron_data_at(
        &self,
        layout: &NeuronLayoutOf<FIQ, NM>,
        index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
    ) -> (NM::NeuronDataInternal, NM::NeuronDataScratch) {
        let internal = self.neuron_internal.clone();
        let scratch = NM::generate_neuron_scratch(index, layout, &internal);
        (internal, scratch)
    }
}

#[cfg(feature = "alloc")]
use feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::from_storage::NeurogenesisByStorage;

impl<FIQ: FeagiIndexQuantization, NM: NeuronModel> NeurogenesisByGenerator<FIQ, NM> for NeurogenesisGeneratorUniformDefault<FIQ, NM> {
    /// The internal data every neuron starts with
    #[cfg(feature = "alloc")]
    type OtherDataNeededForFromStorage = NM::NeuronDataInternal;

    #[cfg(feature = "alloc")]
    fn new_from_storage(storage: &NeurogenesisByStorage<FIQ, NM>, default_neuron_internal: Self::OtherDataNeededForFromStorage) -> Self {
        Self {
            universal_properties: storage.get_universal_cortical_properties().clone(),
            cortical_context: storage.get_cortical_layout_context().clone(),
            cortical_properties: storage.get_model_cortical_properties().clone(),
            cortical_internal: storage.get_model_cortical_internal().clone(),
            cortical_variance: storage.get_model_cortical_variance().clone(),
            neuron_internal: default_neuron_internal,
        }
    }
}
