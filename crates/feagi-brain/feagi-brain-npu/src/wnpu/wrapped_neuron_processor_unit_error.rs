use feagi_logging_and_errors::{generate_feagi_error, FeagiError, FeagiFail};

#[derive(FeagiFail)]
pub struct WNPUFailEtc {
    context: &'static str,
}

generate_feagi_error! {
    WNPUError,
    keys: {
        Etc: WNPUFailEtc,
    },
    sub_errors: {

    },
}
