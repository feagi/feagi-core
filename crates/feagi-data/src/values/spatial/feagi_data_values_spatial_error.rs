use feagi_logging_and_errors::{generate_feagi_error, FeagiError, FeagiErrorKey};

#[derive(FeagiErrorKey)]
/// Attempted to interface with spatial data but used out of range or incorrect quantization
pub struct FeagiFailInvalidSpatialQuantization {
    context: &'static str,
}

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

generate_feagi_error! {
    FeagiDataValuesSpatialError,
    keys: {
        InvalidSpatialQuantization: FeagiFailInvalidSpatialQuantization,
        InvalidSpatialIndex: FeagiFailInvalidSpatialIndex,
        DimensionsCannotBeZero: FeagiFailDimensionsCannotBeZero,
    },
    sub_errors: {

    },
}
