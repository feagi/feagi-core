use feagi_logging_and_errors::{generate_feagi_error, FeagiError, FeagiErrorKey};

#[derive(FeagiErrorKey)]
pub struct FeagiFailNPUEtc {
    context: &'static str,
}

#[derive(FeagiErrorKey)]
pub struct FeagiFailChannelFail {
    context: &'static str,
}

//region Burst Engine

generate_feagi_error! {
    BurstEngineError,
    keys: {
        NPUEtc: FeagiFailNPUEtc,
        Phase: FeagiFailPhase,
        DataCorruption: FeagiFailBurstEngineDataCorruption,
        ChannelFail: FeagiFailChannelFail,
    },
    sub_errors: {
        
    },
}

#[derive(FeagiErrorKey)]
pub struct FeagiFailPhase {
    context: &'static str,
}

#[derive(FeagiErrorKey)]
pub struct FeagiFailBurstEngineDataCorruption {
    context: &'static str,
}

//endregion

generate_feagi_error! {
    BurstEngineWorkerError,
    keys: {
        NPUEtc: FeagiFailNPUEtc,
        ChannelFail: FeagiFailChannelFail,
    },
    sub_errors: {
        EngineError: BurstEngineError
    },
}

generate_feagi_error! {
    BurstEngineWorkerPoolError,
    keys: {
        NPUEtc: FeagiFailNPUEtc,
        ChannelFail: FeagiFailChannelFail,
    },
    sub_errors: {
    },
}

generate_feagi_error! {
    NPUError,
    keys: {
        NPUEtc: FeagiFailNPUEtc,
        ChannelFail: FeagiFailChannelFail,
    },
    sub_errors: {
        BurstEngine: BurstEngineError,
        BurstEngineWorker: BurstEngineWorkerError,
        BurstEngineWorkerPool: BurstEngineWorkerPoolError,
    },
}