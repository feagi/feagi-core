use feagi_logging_and_errors::{generate_feagi_error, FeagiError, FeagiErrorKey};


#[derive(FeagiErrorKey)]
pub struct EngineKernelFail {
    context: &'static str,
}

#[derive(FeagiErrorKey)]
pub struct EngineConnectomeEditFail {
    context: &'static str,
}

#[derive(FeagiErrorKey)]
pub struct EngineFailEtc {
    context: &'static str,
}

generate_feagi_error! {
    FeagiBurstEngineError,
    keys: {
        Kernel: EngineKernelFail,
        ConnectomeEdit: EngineConnectomeEditFail,
        Etc: EngineFailEtc,
    },
    sub_errors: {

    },
}
