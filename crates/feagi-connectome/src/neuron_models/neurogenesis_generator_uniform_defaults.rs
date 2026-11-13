//! A neurogenesis generator starting every neuron of a cortical area from the same values. Any
//! neuron model may use it as its `NeuronModel::NeurogenesisGenerator`. Users of a model, such as
//! cortical types, wrap it to fill in their own defaults. This is a very common generator

use crate::feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::by_generator::NeurogenesisByGenerator;
use crate::feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::{NeuronLayout, NeuronLayoutError, NeuronLayoutRestrictor};
use feagi_basis::neurons::CorticalAreaNeuronLocalIndex;
use feagi_connectome_neural::neural_basis::neuron_model::model_data::NeuronModelUniversalCorticalProperties;
use feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::by_generator::NeurogenesisGenerator;
use feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::from_storage::NeurogenesisStorage;
use feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::NeurogenesisMethod;
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::{CorticalLayoutContextOf, NeuronLayoutOf};
use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
use serde::{Deserialize, Serialize};

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
    layout: NeuronLayoutOf<FIQ, NM>,
    /// Cortical level data used by all models
    universal_properties: NeuronModelUniversalCorticalProperties<NM>,
    cortical_properties: NM::CorticalDataProperties,
    cortical_internal: NM::CorticalDataInternal,
    cortical_variance: NM::CorticalDataVariance,
    /// The internal data every neuron of the area starts with
    neuron_internal: NM::NeuronDataInternal,
}



impl<FIQ: FeagiIndexQuantization, NM: NeuronModel> NeurogenesisMethod<FIQ, NM> for NeurogenesisGeneratorUniformDefault<FIQ, NM> {
    fn get_layout(&self) -> &NeuronLayoutOf<FIQ, NM> {
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
    
}

/// Default Data for a neuron model's init
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct NeurogenesisGeneratorPrecursorUniformDefault<FIQ: FeagiIndexQuantization, NM: NeuronModel> {
    /// Cortical level data used by all models
    pub universal_properties: NeuronModelUniversalCorticalProperties<NM>,
    pub cortical_context: CorticalLayoutContextOf<FIQ, NM>,
    pub cortical_properties: NM::CorticalDataProperties,
    pub cortical_internal: NM::CorticalDataInternal,
    pub cortical_variance: NM::CorticalDataVariance,
    /// The internal data every neuron of the area starts with
    pub neuron_internal: NM::NeuronDataInternal,
}

impl<FIQ, NM> NeurogenesisByGenerator<FIQ, NM> for NeurogenesisGeneratorPrecursorUniformDefault<FIQ, NM>
where 
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel<NeurogenesisGenerator<FIQ> = NeurogenesisGeneratorUniformDefault<FIQ, NM>>
{
    type OtherDataNeededForFromStorage = NM::NeuronDataInternal;

    fn new_from_storage(storage: &NeurogenesisStorage<FIQ, NM>, default_neuron_internal: Self::OtherDataNeededForFromStorage) -> Self {
        
        let (cortical_properties, cortical_internal) = storage.clone_model_cortical_data();
        
        Self {
            universal_properties: storage.get_universal_cortical_properties(),
            cortical_context: storage.get_layout().get_cortical_layout_context().clone(),
            cortical_properties,
            cortical_internal,
            cortical_variance: storage.get_model_cortical_variance().clone(),
            neuron_internal: default_neuron_internal,
        }
    }

    fn into_generator<Res: NeuronLayoutRestrictor<FIQ, NeuronLayout=NeuronLayoutOf<FIQ, NM>>>(self)
        -> Result<NM::NeurogenesisGenerator<FIQ>, NeuronLayoutError> {
        
        let layout = Res::validate_and_create_neuron_layout(self.cortical_context)?;
        
        Ok(NeurogenesisGeneratorUniformDefault {
            layout,
            universal_properties: self.universal_properties,
            cortical_properties: self.cortical_properties,
            cortical_internal: self.cortical_internal,
            cortical_variance: self.cortical_variance,
            neuron_internal: self.neuron_internal,
        })
    }
}