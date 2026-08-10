use feagi_logging_and_errors::{generate_feagi_error, FeagiError, FeagiErrorKey};

#[derive(FeagiErrorKey)]
pub struct FeagiIndexManagerInvalid {
    context: &'static str,
}

#[derive(FeagiErrorKey)]
pub struct FeagiIndexManagerLimit {
    context: &'static str,
}

#[derive(FeagiErrorKey)]
pub struct FeagiIndexManagerInvalidIndex {
    context: &'static str,
    index: usize,
}

#[derive(FeagiErrorKey)]
pub struct FeagiIndexRangeVectorFailedMerge {
    context: &'static str,
}

generate_feagi_error! {
    FeagiIndexRangeManagerError,
    keys: {
        IndexManagerError: FeagiIndexManagerInvalid,
        IndexManagerLimit: FeagiIndexManagerLimit,
        IndexManagerIndex: FeagiIndexManagerInvalidIndex,
        RangeVectorFailedMerge: FeagiIndexRangeVectorFailedMerge,
    },
    sub_errors: {

    }
}
