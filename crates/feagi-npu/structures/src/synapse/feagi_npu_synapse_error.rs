

pub enum FeagiNPUSynapseError {
    SynapseIndexOutOfRange{context: &'static str, given_synapse_index: u32, range: u32},
    SynapseIndexIsInvalid{context: &'static str, given_synapse_index: u32},
    NeuronLookupIndexIsInvalid{context: &'static str, given_neuron_index: u32},
    InternalError{context: &'static str},
}