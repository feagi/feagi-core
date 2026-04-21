use crate::neuron::FeagiNPUNeuronError;
use crate::synapse::FeagiNPUSynapseError;

// TODO how does connectome struct fit here?
pub enum FeagiNPUStructureError {
    NeuronError{error: FeagiNPUNeuronError},
    SynapseError{error: FeagiNPUSynapseError},
}

impl From<FeagiNPUNeuronError> for FeagiNPUStructureError {
    fn from(error: FeagiNPUNeuronError) -> Self {
        Self::NeuronError { error }
    }
}

impl From<FeagiNPUSynapseError> for FeagiNPUStructureError {
    fn from(error: FeagiNPUSynapseError) -> Self {
        Self::SynapseError { error }
    }
}
