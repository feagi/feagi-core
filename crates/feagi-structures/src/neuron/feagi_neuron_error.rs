#[derive(Debug)]
pub enum FeagiNeuronError {
    NeuronIndividualIndexOutOfRange{context: &'static str, given_neuron_index: u32, range: u32},
    IncompatibleNeuronDataFormat{context: &'static str},
    BadParameters{context: &'static str,},
    InternalError{context: &'static str,},
}