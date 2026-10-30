

#[derive(Debug, thiserror::Error)]
pub enum NeuronModelMatchSelectionError {
    #[error("Tried to access this neuron model as a neuron model {tried} when it was actually a neuron model {actual}")]
    IncorrectNeuronModel{tried: &'static str, actual: &'static str},
    #[error("Tried to access this quantization as a quantization {tried} when it was actually a quantization {actual}")]
    IncorrectQuantization{tried: &'static str, actual: &'static str},
}