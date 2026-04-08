

pub enum FeagiNPUSynapseError {
    SynapseIndexOutOfRange{context: &'static str, given_synapse_index: u32, range: u32},
    InternalError{context: &'static str},
}