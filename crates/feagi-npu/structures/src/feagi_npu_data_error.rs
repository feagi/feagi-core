use crate::neuron::FeagiNPUNeuronError;

// TODO how does connectome struct fit here?
pub enum FeagiNPUDataError {
    NeuronError{error: FeagiNPUNeuronError},
}

impl From<FeagiNPUNeuronError> for FeagiNPUDataError {
    fn from(error: FeagiNPUNeuronError) -> Self {
        Self::NeuronError { error }
    }
}
