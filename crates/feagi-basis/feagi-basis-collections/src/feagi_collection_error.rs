use feagi_basis_error_logging::prelude::*;

generate_feagi_error! {
    /// Error related to a collection type within FEAGI
    FeagiDataCollectionError,
    keys: {
        InvalidDimensions: FeagiFailInvalidDimensions,
        InvalidAxisOrder: FeagiFailInvalidAxisOrder,
        ParDataInvalidIndex: ParDataInvalidIndex,
        ParDataInvalidRange: ParDataInvalidRange,
        BitBatchParDataInvalidIndex: BitBatchParDataInvalidIndex,
        BitBatchParDataInvalidRange: BitBatchParDataInvalidRange,
    },
    sub_errors: {},
}

#[derive(FeagiFail)]
/// Dimensions was set to 0 in some axis
pub struct FeagiFailInvalidDimensions {
    context: &'static str,
}

#[derive(FeagiFail)]
/// Axis order is invalid for the given dimension count.
pub struct FeagiFailInvalidAxisOrder {
    context: &'static str,
}

#[derive(FeagiFail)]
/// A generic parallel-data index was out of bounds.
pub struct ParDataInvalidIndex {
    context: &'static str,
    index: usize,
}

#[derive(FeagiFail)]
/// A generic parallel-data sub-range was out of bounds or otherwise invalid.
pub struct ParDataInvalidRange {
    context: &'static str,
    start: usize,
    end: usize,
}

#[derive(FeagiFail)]
/// A bit-batch word index was out of bounds.
pub struct BitBatchParDataInvalidIndex {
    context: &'static str,
    index: usize,
}

#[derive(FeagiFail)]
/// A bit-batch word sub-range was out of bounds or otherwise invalid.
pub struct BitBatchParDataInvalidRange {
    context: &'static str,
    start: usize,
    end: usize,
}