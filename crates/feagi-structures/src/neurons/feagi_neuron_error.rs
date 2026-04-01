#[derive(Debug)]
pub enum FeagiStructuresNeuronError {
    NeuronIndexOutOfRange{context: &'static str, given_neuron_index: usize, range: usize},
    IncompatibleNeuronDataFormat{context: &'static str},
    BadParameters{context: &'static str,},
    InternalError{context: &'static str,},
}