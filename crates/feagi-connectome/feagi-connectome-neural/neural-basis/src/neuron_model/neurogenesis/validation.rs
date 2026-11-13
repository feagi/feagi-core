use crate::neuron_model::neurogenesis::NeurogenesisMethod;
use crate::neuron_model::neuron_layout::{CorticalLayoutContextOf, NeuronLayoutError, NeuronLayoutOf, NeuronLayoutRestrictor};
use crate::neuron_model::NeuronModel;
use crate::quantization_levels::FeagiIndexQuantization;
use crate::neuron_model::model_data::NeuronModelUniversalCorticalProperties;
#[cfg(feature = "alloc")]
use crate::neuron_model::neurogenesis::from_storage::NeurogenesisByStorage;

/// Calls `$f` on whichever validated method the enum holds
#[cfg(feature = "alloc")]
macro_rules! either {
    ($self:ident.$f:ident()) => {
        match $self {
            ValidatedNeurogenesisStorageOrGenerator::Storage(s) => s.$f(),
            ValidatedNeurogenesisStorageOrGenerator::Generator(g) => g.$f(),
        }
    };
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
/// `NeurogenesisValidator` as proof that it is safe to init. Not serializable, save the precursor instead
#[derive(Clone, Debug)]
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
        method_data.check_against_layout(&layout)?;
        Ok(
            ValidatedNeurogenesisMethod {
                method: method_data,
                calculated_neuron_layout: layout,
            }
        )
    }

    pub fn peek_neurogenesis_impl(&self) -> &NGM {
        &self.method
    }

    pub fn peek_layout(&self) -> &NeuronLayoutOf<FIQ, NM> {
        &self.calculated_neuron_layout
    }

    pub fn get_cortical_context(&self) -> &CorticalLayoutContextOf<FIQ, NM> {
        self.method.get_cortical_layout_context()
    }

    pub fn get_universal_properties(&self) -> &NeuronModelUniversalCorticalProperties<NM> {
        self.method.get_universal_cortical_properties()
    }

    pub fn get_model_cortical_properties(&self) -> &NM::CorticalDataProperties {
        self.method.get_model_cortical_properties()
    }

    pub fn get_model_cortical_variance(&self) -> &NM::CorticalDataVariance {
        self.method.get_model_cortical_variance()
    }

    /// Unwrap the verified method to get back the original struct and layout
    pub fn unwrap(self) -> (NGM, NeuronLayoutOf<FIQ, NM>) {
        (self.method, self.calculated_neuron_layout)
    }
}

#[cfg(feature = "alloc")]
pub type ValidatedNeurogenesisStorage<FIQ: FeagiIndexQuantization, NM: NeuronModel> = ValidatedNeurogenesisMethod<FIQ, NM, NeurogenesisByStorage<FIQ, NM>>;
pub type ValidatedNeurogenesisGenerator<FIQ: FeagiIndexQuantization, NM: NeuronModel> = ValidatedNeurogenesisMethod<FIQ, NM, NM::NeurogenesisByGenerator<FIQ>>;

#[cfg(feature = "alloc")]
#[derive(Clone, Debug)]
pub enum ValidatedNeurogenesisStorageOrGenerator<FIQ: FeagiIndexQuantization, NM: NeuronModel> {
    Storage(ValidatedNeurogenesisStorage<FIQ, NM>),
    Generator(ValidatedNeurogenesisGenerator<FIQ, NM>)
}

#[cfg(feature = "alloc")]
impl<FIQ, NM> ValidatedNeurogenesisStorageOrGenerator<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
{
    pub fn peek_layout(&self) -> &NeuronLayoutOf<FIQ, NM> {
        either!(self.peek_layout())
    }

    pub fn get_cortical_context(&self) -> &CorticalLayoutContextOf<FIQ, NM> {
        either!(self.get_cortical_context())
    }

    pub fn get_universal_properties(&self) -> &NeuronModelUniversalCorticalProperties<NM> {
        either!(self.get_universal_properties())
    }

    pub fn get_model_cortical_properties(&self) -> &NM::CorticalDataProperties {
        either!(self.get_model_cortical_properties())
    }

    pub fn get_model_cortical_variance(&self) -> &NM::CorticalDataVariance {
        either!(self.get_model_cortical_variance())
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