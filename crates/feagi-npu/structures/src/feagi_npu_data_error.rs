
pub enum FeagiNPUDataError {
    NeuronIndexOutOfRange{given_neuron_index: u32, range: u32},
    InvalidCorticalIndex{given_cortical_index: u32},
    InternalError(),
}
