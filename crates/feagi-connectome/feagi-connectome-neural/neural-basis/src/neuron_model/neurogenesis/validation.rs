use crate::neuron_model::neurogenesis::NeurogenesisMethod;
use crate::neuron_model::neuron_layout::{CorticalLayoutContextOf, NeuronLayoutError, NeuronLayoutOf, NeuronLayoutRestrictor};
use crate::neuron_model::NeuronModel;
use crate::quantization_levels::FeagiIndexQuantization;
use serde::{Deserialize, Serialize};
use crate::neuron_model::model_data::NeuronModelUniversalCorticalProperties;
use crate::neuron_model::neurogenesis::from_storage::NeurogenesisByStorage;

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


pub trait ValidatedNeurogenesisPeeking<FIQ: FeagiIndexQuantization, NM: NeuronModel> {

    fn peek_neurogenesis_impl(&self) -> &impl NeurogenesisMethod<FIQ, NM>;

    fn peek_layout(&self) -> &NeuronLayoutOf<FIQ, NM>;
    
    
    //region Default Impls
    
    fn get_cortical_context(&self) -> &CorticalLayoutContextOf<FIQ, NM> {
        self.peek_neurogenesis_impl().get_cortical_layout_context()
    }
    
    fn get_universal_properties(&self) -> &NeuronModelUniversalCorticalProperties<NM> {
        self.peek_neurogenesis_impl().get_universal_cortical_properties()
    }

    fn get_model_cortical_properties(&self) -> &NM::CorticalDataProperties {
        self.peek_neurogenesis_impl().get_model_cortical_properties()
    }

    fn get_model_cortical_variance(&self) -> &NM::CorticalDataVariance {
        self.peek_neurogenesis_impl().get_model_cortical_variance()
    }
    //endregion
    
}


/// A wrapped implementation of `NeurogenesisMethod` that can only be wrapped by
/// `NeurogenesisValidator` as proof that it is safe to init
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct ValidatedNeurogenesisMethod<FIQ, NM, NGM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
    NGM: NeurogenesisMethod<FIQ, NM>
{
    method: NGM,
    calculated_neuron_layout: NeuronLayoutOf<FIQ, NM>
}

impl<FIQ, NM, NGM> ValidatedNeurogenesisMethod<FIQ, NM, NGM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
    NGM: NeurogenesisMethod<FIQ, NM>
{
    pub fn validate_and_wrap<
        V: NeurogenesisValidator<FIQ, NM>,
        Res: NeuronLayoutRestrictor<FIQ, NeuronLayout=NeuronLayoutOf<FIQ, NM>>
    >(method_data: NGM) -> Result<Self, NeurogenesisValidationError> {
        let layout = Res::validate_and_create_neuron_layout(method_data.get_cortical_layout_context().clone())?;
        V::validate(&method_data)?;
        Ok(
            ValidatedNeurogenesisMethod {
                method: method_data,
                calculated_neuron_layout: layout,
            }
        )
    }

    /// Unwrap the verified method to get back the original struct and layout
    pub fn unwrap(self) -> (NGM, NeuronLayoutOf<FIQ, NM>) {
        (self.method, self.calculated_neuron_layout)
    }
}

impl<FIQ, NM, NGM> ValidatedNeurogenesisPeeking<FIQ, NM> for ValidatedNeurogenesisMethod<FIQ, NM, NGM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
    NGM: NeurogenesisMethod<FIQ, NM>
{
    fn peek_neurogenesis_impl(&self) -> &impl NeurogenesisMethod<FIQ, NM> {
        &self.method
    }

    fn peek_layout(&self) -> &NeuronLayoutOf<FIQ, NM> {
        &self.calculated_neuron_layout
    }
}


pub type ValidatedNeurogenesisStorage<FIQ: FeagiIndexQuantization, NM: NeuronModel> = ValidatedNeurogenesisMethod<FIQ, NM, NeurogenesisByStorage<FIQ, NM>>;
pub type ValidatedNeurogenesisGenerator<FIQ: FeagiIndexQuantization, NM: NeuronModel> = ValidatedNeurogenesisMethod<FIQ, NM, NM::NeurogenesisByGenerator<FIQ>>;

pub enum ValidatedNeurogenesisStorageOrGenerator<FIQ: FeagiIndexQuantization, NM: NeuronModel> {
    Storage(ValidatedNeurogenesisGenerator<FIQ, NM>),
    Generator(ValidatedNeurogenesisGenerator<FIQ, NM>)
}

impl<FIQ, NM> ValidatedNeurogenesisPeeking<FIQ, NM> for ValidatedNeurogenesisStorageOrGenerator<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
{
    fn peek_neurogenesis_impl(&self) -> &impl NeurogenesisMethod<FIQ, NM> {
        match &self {
            ValidatedNeurogenesisStorageOrGenerator::Storage(s) => {s.peek_neurogenesis_impl()}
            ValidatedNeurogenesisStorageOrGenerator::Generator(g) => {g.peek_neurogenesis_impl()}
        }
    }

    fn peek_layout(&self) -> &NeuronLayoutOf<FIQ, NM> {
        match &self {
            ValidatedNeurogenesisStorageOrGenerator::Storage(s) => {s.peek_layout()}
            ValidatedNeurogenesisStorageOrGenerator::Generator(g) => {g.peek_layout()}
        }
    }
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