use feagi_logging_and_errors::{generate_feagi_error, FeagiError, FeagiErrorKey};
use crate::values::quantizable::FeagiDataValueQuantizationError;

#[derive(FeagiErrorKey)]
/// Attempted to index using a coordinate but it was not in the given dimensions
pub struct FeagiFailInvalidSpatialIndex {
    context: &'static str,
}

#[derive(FeagiErrorKey)]
/// Attempted to create a dimensions value with a zero sized axis
pub struct FeagiFailDimensionsCannotBeZero {
    context: &'static str,
}

#[derive(FeagiErrorKey)]
/// Attempted to convert spatial data to another quantization but a value would not fit in the target quantization
pub struct FeagiFailSpatialQuantizationOutOfRange {
    context: &'static str,
}

generate_feagi_error! {
    FeagiDataValuesSpatialError,
    keys: {
        InvalidSpatialIndex: FeagiFailInvalidSpatialIndex,
        DimensionsCannotBeZero: FeagiFailDimensionsCannotBeZero,
        SpatialQuantizationOutOfRange: FeagiFailSpatialQuantizationOutOfRange,
    },
    sub_errors: {
        InvalidSpatialQuantization: FeagiDataValueQuantizationError
    },
}
