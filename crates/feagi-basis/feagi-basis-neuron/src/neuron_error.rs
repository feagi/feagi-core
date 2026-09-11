use feagi_basis_error_logging::prelude::*;

#[derive(FeagiFail)]
/// A bit or u32-word index was out of bounds for a neuron activation collection.
pub struct FeagiNeuronActivationInvalidIndex {
    context: &'static str,
    index: u64,
}

#[derive(FeagiFail)]
/// A u32-word sub-range was out of bounds or otherwise invalid (e.g. start > end).
pub struct FeagiNeuronActivationInvalidRange {
    context: &'static str,
    start: u64,
    end: u64,
}

#[derive(FeagiFail)]
/// The declared number of addressable activation bits exceeds the backing u32 storage.
pub struct FeagiNeuronActivationInvalidBitCount {
    context: &'static str,
    addressable_bits: u64,
    capacity_bits: u64,
}

#[derive(FeagiFail)]
pub struct FeagiNeuronInvalidNeuronIndex {
    context: &'static str,
}

generate_feagi_error! {
    FeagiNeuronError,
    keys: {
        InvalidIndex: FeagiNeuronInvalidNeuronIndex
    },
    sub_errors: {

    },
}
