use feagi_basis_error_logging::prelude::*;

#[derive(FeagiFail)]
/// Dimensions was set to 0 in some axis
pub struct FeagiFailInvalidDimensions {
    context: &'static str,
}


generate_feagi_error!{
    /// Error related to a collection type within FEAGI
    FeagiDataCollectionError,
    keys: {
        InvalidDimensions: FeagiFailInvalidDimensions,
    },
    sub_errors: {},
}