use feagi_data::data_channels::errors::ChannelSendingError;
use feagi_logging_and_errors::{generate_feagi_error, FeagiError, FeagiFailImpossible};
use feagi_npu_burst_core::errors::{BurstEngineError};

generate_feagi_error! {
    BurstEngineWorkerError,
    keys: {
        Impossible: FeagiFailImpossible,
    },
    sub_errors: {
        EngineError: BurstEngineError,
        ChannelSendError: ChannelSendingError,
    },
}

generate_feagi_error! {
    BurstEngineWorkerPoolError,
    keys: {
        Impossible: FeagiFailImpossible,
    },
    sub_errors: {
    },
}

generate_feagi_error! {
    NPUError,
    keys: {
        Impossible: FeagiFailImpossible,
    },
    sub_errors: {
        BurstEngine: BurstEngineError,
        BurstEngineWorker: BurstEngineWorkerError,
        BurstEngineWorkerPool: BurstEngineWorkerPoolError,
    },
}