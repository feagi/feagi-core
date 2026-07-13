use feagi_logging_and_errors::{generate_feagi_error, FeagiError, FeagiErrorKey};

#[derive(FeagiErrorKey)]
pub struct FeagiInvalidQuantizationErrKey {
    context: &'static str,
}

#[derive(FeagiErrorKey)]
pub struct FeagiInvalidIndexErrKey {
    context: &'static str,
}

#[derive(FeagiErrorKey)]
pub struct FeagiInvalidCoordErrKey {
    context: &'static str,
}

#[derive(FeagiErrorKey)]
pub struct FeagiDimensionsErrKey {
    context: &'static str,
}

generate_feagi_error! {
    FeagiValueError,
    keys: {
        InvalidQuantization: FeagiInvalidQuantizationErrKey,
        InvalidIndex: FeagiInvalidIndexErrKey,
        InvalidCoordinate: FeagiInvalidCoordErrKey,
        InvalidDimensions: FeagiDimensionsErrKey
    },
    sub_errors: {

    },
}
