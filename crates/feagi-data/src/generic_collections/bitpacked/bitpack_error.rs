use feagi_logging_and_errors::{generate_feagi_error, FeagiError, FeagiErrorKey};

#[derive(FeagiErrorKey)]
/// A bit or u32-word index was out of bounds
pub struct BitPackInvalidIndex {
    context: &'static str,
    index: usize,
}

#[derive(FeagiErrorKey)]
/// A u32-word sub-range was out of bounds or otherwise invalid (e.g. start > end).
pub struct BitPackInvalidRange {
    context: &'static str,
    start: usize,
    end: usize,
}

#[derive(FeagiErrorKey)]
/// The declared number of addressable activation bits exceeds the backing u32 storage.
pub struct BitPackInvalidBitCount {
    context: &'static str,
    addressable_bits: usize,
    capacity_bits: usize,
}

generate_feagi_error! {
    /// Error related to u32 bit-packed neuron activation collections.
    BitPackError,
    keys: {
        InvalidIndex: BitPackInvalidIndex,
        InvalidRange: BitPackInvalidRange,
        InvalidBitCount: BitPackInvalidBitCount,
    },
    sub_errors: {},
}
