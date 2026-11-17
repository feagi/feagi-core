use crate::neuron_model::neurogenesis::NeurogenesisMethod;
use crate::neuron_model::neuron_layout::NeuronLayoutError;
use crate::neuron_model::NeuronModel;
use crate::quantization_levels::FeagiIndexQuantization;

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


#[derive(Debug, thiserror::Error, Clone, Copy, Hash)]
pub enum NeurogenesisValidationError {
    #[error("Invalid neuron layout: {0}")]
    Layout(#[from] NeuronLayoutError),
    #[error("The stored neuron data does not match the size of the layout!")]
    StorageSizeMismatch,
    #[error("Rejected by validator: {0}")]
    Rejected(&'static str),
}