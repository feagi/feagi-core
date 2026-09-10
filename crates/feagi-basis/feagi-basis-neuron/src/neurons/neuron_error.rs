use feagi_logging_and_errors::{generate_feagi_error, FeagiError, FeagiFail};

#[derive(FeagiFail)]
/// A bit or u32-word index was out of bounds for a neuron activation collection.
pub struct FeagiNeuronActivationInvalidIndex {
    context: &'static str,
    index: usize,
}

#[derive(FeagiFail)]
/// A u32-word sub-range was out of bounds or otherwise invalid (e.g. start > end).
pub struct FeagiNeuronActivationInvalidRange {
    context: &'static str,
    start: usize,
    end: usize,
}

#[derive(FeagiFail)]
/// The declared number of addressable activation bits exceeds the backing u32 storage.
pub struct FeagiNeuronActivationInvalidBitCount {
    context: &'static str,
    addressable_bits: usize,
    capacity_bits: usize,
}

#[derive(FeagiFail)]
pub struct FeagiNeuronInvalidNeuronIndex {
    context: &'static str,
}

generate_feagi_error! {
    FeagiVoxelError,
    keys: {
        InvalidIndex: FeagiNeuronInvalidNeuronIndex
    },
    sub_errors: {

    },
}
