use feagi_basis_error_logging::prelude::*;

#[derive(FeagiFail)]
pub struct FeagiIndexManagerInvalid {
    context: &'static str,
}

#[derive(FeagiFail)]
pub struct FeagiIndexManagerLimit {
    context: &'static str,
}

#[derive(FeagiFail)]
pub struct FeagiIndexManagerInvalidIndex {
    context: &'static str,
    index: usize,
}

#[derive(FeagiFail)]
pub struct FeagiIndexRangeVectorFailedMerge {
    context: &'static str,
}

generate_feagi_error! {
    FeagiIndexOrganizerError,
    keys: {
        IndexManagerError: FeagiIndexManagerInvalid,
        IndexManagerLimit: FeagiIndexManagerLimit,
        IndexManagerIndex: FeagiIndexManagerInvalidIndex,
        RangeVectorFailedMerge: FeagiIndexRangeVectorFailedMerge,
    },
    sub_errors: {

    }
}
