use feagi_connectome::neuron_models::invalid_neuron_enum_error::NeuronModelMatchSelectionError;

#[derive(Debug, thiserror::Error)]
pub enum CorticalTypeMatchSelectionError {
    #[error("Tried to access this cortical supertype as a supertype {tried} when it was actually a supertype {actual}")]
    IncorrectCorticalSuperType{tried: &'static str, actual: &'static str},
    #[error("Tried to access this cortical type as a type {tried} when it was actually a type {actual}")]
    IncorrectCorticalType{tried: &'static str, actual: &'static str},
    #[error(transparent)]
    NeuronModel(#[from] NeuronModelMatchSelectionError),
}