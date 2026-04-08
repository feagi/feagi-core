use crate::neuron::FeagiNPUNeuronError;

// TODO how does connectome struct fit here?
pub enum FeagiNPUStructureError {
    NeuronError{error: FeagiNPUNeuronError},
}

impl From<FeagiNPUNeuronError> for FeagiNPUStructureError {
    fn from(error: FeagiNPUNeuronError) -> Self {
        Self::NeuronError { error }
    }
}
