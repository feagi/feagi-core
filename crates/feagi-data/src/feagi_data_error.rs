use feagi_logging_and_errors::{generate_feagi_error, FeagiError, FeagiErrorKey};

/// A Genric error type. Anything using this should be updated with more specific errors
#[derive(FeagiErrorKey)]
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


#[derive(FeagiErrorKey)]
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