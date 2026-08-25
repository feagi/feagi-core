use feagi_logging_and_errors::{generate_feagi_error, FeagiError, FeagiErrorKey};

#[derive(FeagiErrorKey)]
pub struct FeagiVoxelsInvalidDimensions {
    context: &'static str,
}

#[derive(FeagiErrorKey)]
pub struct FeagiNeuronInvalidLinearIndex {
    context: &'static str,
}

#[derive(FeagiErrorKey)]
pub struct FeagiNeuronInvalidCoordinate {
    context: &'static str,
}


generate_feagi_error! {
    FeagiVoxelError,
    keys: {
        InvalidDimensions: FeagiVoxelsInvalidDimensions,
        InvalidLinearIndex: FeagiNeuronInvalidLinearIndex,
        InvalidCoordinate: FeagiNeuronInvalidCoordinate
    },
    sub_errors: {

    },
}
