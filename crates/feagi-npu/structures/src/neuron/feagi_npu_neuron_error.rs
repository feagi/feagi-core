

pub enum FeagiNPUNeuronError {
    NeuronIndexOutOfRange{context: &'static str, given_neuron_index: u32, range: u32},
    InvalidCorticalIndex{context: &'static str, given_cortical_index: u32},
    InternalError{context: &'static str},
}