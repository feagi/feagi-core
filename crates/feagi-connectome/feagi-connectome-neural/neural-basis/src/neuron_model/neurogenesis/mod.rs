pub mod from_storage;
pub mod by_generator;
pub mod scratch_variance_defaults;

use crate::neuron_model::model_data::NeuronModelUniversalCorticalProperties;
use crate::neuron_model::neuron_layout::{CorticalLayoutContextOf, NeuronLayoutError, NeuronLayoutOf, NeuronLayoutRestrictor, NeuronLayoutTag};
use crate::neuron_model::NeuronModel;
use crate::quantization_levels::FeagiIndexQuantization;
use feagi_basis::neurons::CorticalAreaNeuronLocalIndex;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use crate::neuron_model::neurogenesis::from_storage::{NeurogenesisByStorage};

/// How a cortical area gets spawned, implemented by every precursor (the saveable form). Neurons
/// are only spawned through [`NeurogenesisValidated`](validation::NeurogenesisValidated), which provides the validated layout
pub trait NeurogenesisMethod<FIQ, NM>: Clone + core::fmt::Debug + Send + Sync + Serialize + DeserializeOwned
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel
{
    /// The layout context, not yet validated
    fn get_cortical_layout_context(&self) -> &CorticalLayoutContextOf<FIQ, NM>;

    /// Cortical level data used by all models
    fn get_universal_cortical_properties(&self) -> NeuronModelUniversalCorticalProperties<NM>;

    fn get_model_cortical_properties(&self) -> &NM::CorticalDataProperties;

    fn get_model_cortical_internal(&self) -> &NM::CorticalDataInternal;

    fn get_model_cortical_variance(&self) -> &NM::CorticalDataVariance;

    fn into_model_cortical_data(self) -> (NM::CorticalDataProperties, NM::CorticalDataInternal);


    fn spawn_neuron_data_at(
        &self, // NOTE: make sure the implementation makes use of variance!
        layout: &NeuronLayoutOf<FIQ, NM>,
        index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
    ) -> (NM::NeuronDataInternal, NM::NeuronDataScratch);

}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum NeurogenesisStorageOrGenerator<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
{
    Storage(NeurogenesisByStorage<FIQ, NM>),
    Generator(NM::NeurogenesisByGenerator<FIQ>),
}



/// Checks a precursor before it can be spawned. Implemented on zero sized structs, typically one per
/// cortical type
pub trait NeurogenesisValidator<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
{
    /// Any further checks, run after the layout is built. Accepts everything by default
    fn validate<P: NeurogenesisMethod<FIQ, NM>>(_precursor: &P) -> Result<(), NeurogenesisValidationError> {
        // default impl
        Ok(())
    }
}


/// A wrapped implementation of `NeurogenesisMethod` that can only be wrapped by
/// `NeurogenesisValidator` as proof that it is safe to init
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct VerifiedNeurogenesisMethod<FIQ, NM, NGM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
    NGM: NeurogenesisMethod<FIQ, NM>
{
    method: NGM,
    calculated_neuron_layout: NeuronLayoutOf<FIQ, NM>
}

impl<FIQ, NM, NGM> VerifiedNeurogenesisMethod<FIQ, NM, NGM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
    NGM: NeurogenesisMethod<FIQ, NM>
{
    pub(crate) fn validate_and_wrap<
        V: NeurogenesisValidator<FIQ, NM>,
        Res: NeuronLayoutRestrictor<FIQ, NeuronLayout=NeuronLayoutOf<FIQ, NM>>
    >(method_data: NGM) -> Result<Self, NeurogenesisValidationError> {
        let layout = Res::validate_and_create_neuron_layout(method_data.get_cortical_layout_context().clone())?;
        V::validate(&method_data)?;
        Ok(
            VerifiedNeurogenesisMethod {
                method: method_data,
                calculated_neuron_layout: layout,
            }
        )
    }

    /// Unwrap the verified method to get back the original struct and layout
    pub fn unwrap(self) -> (NGM, NeuronLayoutOf<FIQ, NM>) {
        (self.method, self.calculated_neuron_layout)
    }

    // TODO methods to peek some needed data (genome properties, universal, layout (neuron coords ))

    // TODO to do in place updates for genome level data for Genome to use without needing to reinit. Only allow inplace ops
}


#[derive(Debug, thiserror::Error, Clone, Copy, Hash)]
pub enum NeurogenesisValidationError {
    #[error("Invalid neuron layout: {0}")]
    Layout(#[from] NeuronLayoutError),
    #[error("The stored neuron data does not match the size of the layout!")]
    StorageSizeMismatch,
    #[error("Rejected by validator: {0}")]
    Rejected(&'static str),
}