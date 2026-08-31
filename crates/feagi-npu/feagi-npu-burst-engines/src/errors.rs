use feagi_data::data_channels::errors::ChannelSendingError;
use feagi_logging_and_errors::{generate_feagi_error, FeagiError, FeagiErrorKey};
use feagi_npu_burst_core::errors::{BurstEngineError, FeagiFailNPUInvalidArguments};

generate_feagi_error! {
    BurstEngineWorkerError,
    keys: {
        NPUInvalidArguments: FeagiFailNPUInvalidArguments,
    },
    sub_errors: {
        EngineError: BurstEngineError,
        ChannelSendError: ChannelSendingError,
    },
}

generate_feagi_error! {
    BurstEngineWorkerPoolError,
    keys: {
        NPUInvalidArguments: FeagiFailNPUInvalidArguments,
    },
    sub_errors: {
    },
}

generate_feagi_error! {
    NPUError,
    keys: {
        NPUInvalidArguments: FeagiFailNPUInvalidArguments,
    },
    sub_errors: {
        BurstEngine: BurstEngineError,
        BurstEngineWorker: BurstEngineWorkerError,
        BurstEngineWorkerPool: BurstEngineWorkerPoolError,
    },
}