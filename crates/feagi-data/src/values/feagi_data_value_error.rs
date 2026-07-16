use feagi_logging_and_errors::{generate_feagi_error, FeagiError, FeagiErrorKey};
use crate::values::quantizable::feagi_data_value_quantization_error::FeagiDataValueQuantizationError;

generate_feagi_error! {
    /// Error related to a base data type of FEAGI
    FeagiDataValueError,
    keys: {

    },
    sub_errors: {
        Quantization: FeagiDataValueQuantizationError
    },
}
