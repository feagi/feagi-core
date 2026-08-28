use feagi_logging_and_errors::{generate_feagi_error, FeagiError, FeagiErrorKey};

#[derive(FeagiErrorKey)]
pub struct FeagiFailNPUEtc {
    context: &'static str,
}

//region Channels

generate_feagi_error! {
    ChannelError,
    keys: {
        SendFailed: FeagiFailChannelSendFailed,
        SendChannelFull: FeagiFailChannelSendFull,
        ReceiveFailed: FeagiFailChannelReceiveFailed,
        ReceiveChannelEmpty: FeagiFailChannelReceiveEmpty,
        SendTimeout: FeagiFailChannelSendTimeout,
        ReceiveTimeout: FeagiFailChannelReceiveTimeout,
    },
    sub_errors: {
        
    },
}

#[derive(FeagiErrorKey)]
pub struct FeagiFailChannelSendFailed {
    context: &'static str,
}

#[derive(FeagiErrorKey)]
pub struct FeagiFailChannelSendFull {
    context: &'static str,
}

#[derive(FeagiErrorKey)]
pub struct FeagiFailChannelReceiveFailed {
    context: &'static str,
}

#[derive(FeagiErrorKey)]
pub struct FeagiFailChannelReceiveEmpty {
    context: &'static str,
}

#[derive(FeagiErrorKey)]
pub struct FeagiFailChannelSendTimeout {
    context: &'static str,
}

#[derive(FeagiErrorKey)]
pub struct FeagiFailChannelReceiveTimeout {
    context: &'static str,
}

#[derive(FeagiErrorKey)]
pub struct FeagiFailThreadChannelTimeout {
    context: &'static str,
    // TODO duration?
}


//endregion

//region Burst Engine

generate_feagi_error! {
    BurstEngineError,
    keys: {
        NPUEtc: FeagiFailNPUEtc,
        Phase: FeagiFailPhase,
        DataCorruption: FeagiFailBurstEngineDataCorruption
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
        ThreadChannelTimeout: FeagiFailThreadChannelTimeout,
    },
    sub_errors: {
        Channel: ChannelError
    },
}

generate_feagi_error! {
    BurstEngineWorkerPoolError,
    keys: {
        NPUEtc: FeagiFailNPUEtc,
        ThreadChannelTimeout: FeagiFailThreadChannelTimeout,
    },
    sub_errors: {
        Channel: ChannelError
    },
}

generate_feagi_error! {
    NPUError,
    keys: {
        NPUEtc: FeagiFailNPUEtc,
    },
    sub_errors: {
        BurstEngine: BurstEngineError,
        BurstEngineWorker: BurstEngineWorkerError,
        BurstEngineWorkerPool: BurstEngineWorkerPoolError,
    },
}