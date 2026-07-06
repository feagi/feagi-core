use feagi_logging_and_errors::{FeagiErrorKey, FeagiError, generate_feagi_error};

#[derive(FeagiErrorKey)]
pub struct FeagiCorticalIDErrKey {
    context: &'static str
}

#[derive(FeagiErrorKey)]
pub struct FeagiCorticalTypeErrKey {
    context: &'static str
}

#[derive(FeagiErrorKey)]
pub struct FeagiCorticalConfigurationFlagErrKey {
    context: &'static str
}

generate_feagi_error!{
    FeagiGenomeContextError,
    keys: {
        CorticalID: FeagiCorticalIDErrKey,
        CorticalType: FeagiCorticalTypeErrKey,
        ConfigurationFlag: FeagiCorticalConfigurationFlagErrKey,
    },
    sub_errors: {

    },
}