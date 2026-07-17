use feagi_logging_and_errors::{generate_feagi_error, FeagiErrorKey, FeagiError};
use crate::values::spatial::feagi_data_values_spatial_error::FeagiDataValuesSpatialError;

#[derive(FeagiErrorKey)]
/// An invalid index (such as one out of range) was used
pub struct FeagiFailCollectionInvalidIndex {
    context: &'static str,
}

#[derive(FeagiErrorKey)]
/// Memory allocation or other operation attempted that does not fit the dimensions of the given struct
pub struct FeagiFailCollectionDimensionMismatch {
    context: &'static str,
}

generate_feagi_error! {
    /// An error related to some quantizable collection for FEAGI
    FeagiDataCollectionError,
    keys: {
        PackedIndex: FeagiFailCollectionInvalidIndex,
        DimensionMismatch: FeagiFailCollectionDimensionMismatch,
    },
    sub_errors: {
        ValueSpatialError: FeagiDataValuesSpatialError,
    },
}
