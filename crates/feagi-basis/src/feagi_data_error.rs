use feagi_logging_and_errors::{generate_feagi_error, FeagiError, FeagiFail};

/// A Genric error type. Anything using this should be updated with more specific errors
#[derive(FeagiFail)]
pub struct FeagiFailDataEtc {
    context: &'static str,
}

generate_feagi_error! {
    /// A Generic Error type
    FeagiDataError,
    keys: {
        DataEtc: FeagiFailDataEtc,
    },
    sub_errors: {

    },
}

/*


#[derive(FeagiFail)]
pub struct FeagiFailDataEtc {
    context: &'static str,
}

generate_feagi_error! {
    FeagiDataError,
    keys: {
        DataEtc: FeagiFailDataEtc,
    },
    sub_errors: {
        QuantizationValue: FeagiDataValueQuantizationError,
        SpatialValue: FeagiDataValuesSpatialError,

    },
}

 */