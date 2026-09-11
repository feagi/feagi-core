use feagi_logging_and_errors::{generate_feagi_error, FeagiError, FeagiFail};

#[derive(FeagiFail)]
pub struct FeagiCorticalIDErrKey {
    context: &'static str,
}

#[derive(FeagiFail)]
pub struct FeagiCorticalTypeErrKey {
    context: &'static str,
}

#[derive(FeagiFail)]
pub struct FeagiCorticalConfigurationFlagErrKey {
    context: &'static str,
}

#[derive(FeagiFail)]
pub struct FeagiBrainRegionErrKey {
    context: &'static str,
}

generate_feagi_error! {
    FeagiGenomeContextError,
    keys: {
        CorticalID: FeagiCorticalIDErrKey,
        CorticalType: FeagiCorticalTypeErrKey,
        ConfigurationFlag: FeagiCorticalConfigurationFlagErrKey,
        BrainRegion: FeagiBrainRegionErrKey,
    },
    sub_errors: {

    },
}
