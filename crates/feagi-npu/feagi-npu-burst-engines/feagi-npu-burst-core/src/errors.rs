use feagi_logging_and_errors::{generate_feagi_error, FeagiError, FeagiErrorKey};

#[derive(FeagiErrorKey)]
pub struct FeagiFailNPUInvalidArguments {
    context: &'static str,
}



//region Burst Engine

generate_feagi_error! {
    BurstEngineError,
    keys: {
        NPUInvalidArguments: FeagiFailNPUInvalidArguments,
        Phase: FeagiFailPhase,
        DataCorruption: FeagiFailConnectomeCorruption,
    },
    sub_errors: {
        
    },
}

#[derive(FeagiErrorKey)]
pub struct FeagiFailPhase {
    context: &'static str,
}

#[derive(FeagiErrorKey)]
pub struct FeagiFailConnectomeCorruption {
    context: &'static str,
}

//endregion
