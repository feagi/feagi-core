
use crate::values::quantizable::FeagiDataValueQuantizationError;
use crate::values::spatial::feagi_data_values_spatial_error::FeagiDataValuesSpatialError;
use feagi_logging_and_errors::{generate_feagi_error, FeagiError, FeagiErrorKey};

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
