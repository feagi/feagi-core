use feagi_basis_error_logging::prelude::*;

#[derive(FeagiFail)]
/// A bit or u32-word index was out of bounds
pub struct ParDataInvalidIndex {
    context: &'static str,
    index: usize,
}

#[derive(FeagiFail)]
/// An sub-range was out of bounds or otherwise invalid (e.g. start > end).
pub struct ParDataInvalidRange {
    context: &'static str,
    start: usize,
    end: usize,
}

generate_feagi_error! {
    /// Error related to u32 bit-packed neuron activation collections.
    ParDataError,
    keys: {
        InvalidIndex: ParDataInvalidIndex,
        InvalidRange: ParDataInvalidRange,
    },
    sub_errors: {},
}
