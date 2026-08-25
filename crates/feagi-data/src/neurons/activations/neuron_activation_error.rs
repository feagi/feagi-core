use feagi_logging_and_errors::{generate_feagi_error, FeagiError, FeagiErrorKey};

#[derive(FeagiErrorKey)]
/// A bit or u32-word index was out of bounds for a neuron activation collection.
pub struct FeagiNeuronActivationInvalidIndex {
    context: &'static str,
    index: usize,
}

#[derive(FeagiErrorKey)]
/// A u32-word sub-range was out of bounds or otherwise invalid (e.g. start > end).
pub struct FeagiNeuronActivationInvalidRange {
    context: &'static str,
    start: usize,
    end: usize,
}

#[derive(FeagiErrorKey)]
/// The declared number of addressable activation bits exceeds the backing u32 storage.
pub struct FeagiNeuronActivationInvalidBitCount {
    context: &'static str,
    addressable_bits: usize,
    capacity_bits: usize,
}

generate_feagi_error! {
    /// Error related to u32 bit-packed neuron activation collections.
    FeagiNeuronActivationError,
    keys: {
        InvalidIndex: FeagiNeuronActivationInvalidIndex,
        InvalidRange: FeagiNeuronActivationInvalidRange,
        InvalidBitCount: FeagiNeuronActivationInvalidBitCount,
    },
    sub_errors: {},
}
