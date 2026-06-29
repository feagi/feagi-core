use feagi_logging_and_errors::{FeagiErrorKey, FeagiError, generate_feagi_error};

#[derive(FeagiErrorKey)]
pub struct FeagiInvalidCoordErrKey {
    context: &'static str
}

#[derive(FeagiErrorKey)]
pub struct FeagiDimensionsErrKey {
    context: &'static str
}

generate_feagi_error!{
    FeagiValueError,
    keys: {
        InvalidCoordinate: FeagiInvalidCoordErrKey,
        InvalidDimensions: FeagiDimensionsErrKey
    },
    sub_errors: {

    },
}
