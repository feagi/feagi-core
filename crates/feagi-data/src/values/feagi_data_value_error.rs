use feagi_logging_and_errors::{generate_feagi_error, FeagiError, FeagiErrorKey};
use crate::values::quantizable::feagi_data_value_quantization_error::FeagiDataValueQuantizationError;

#[derive(FeagiErrorKey)]
/// Attempted to index using a coordinate and it was not in the given dimensions
pub struct FeagiFailInvalidSpatialIndex {
    context: &'static str,
}

#[derive(FeagiErrorKey)]
/// Attempted to create a dimensions value with a zero sized axis
pub struct FeagiFailDimensionsCannotBeZero {
    context: &'static str,
}

generate_feagi_error! {
    /// Error related to a base data type of FEAGI
    FeagiDataValueError,
    keys: {

    },
    sub_errors: {
        Quantization: FeagiDataValueQuantizationError
    },
}
