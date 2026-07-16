use feagi_logging_and_errors::{generate_feagi_error, FeagiError, FeagiErrorKey};

#[derive(FeagiErrorKey)]
pub struct FeagiVoxelsInvalidDimensions {
    context: &'static str,
}

generate_feagi_error! {
    FeagiVoxelError,
    keys: {
        InvalidDimensions: FeagiVoxelsInvalidDimensions
    },
    sub_errors: {
        
    },
}
