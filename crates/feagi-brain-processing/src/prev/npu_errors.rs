use feagi_basis::prelude::*;


generate_feagi_error! {
    BurstEngineWorkerError,
    keys: {
        Impossible: FeagiFailImpossible,
    },
    sub_errors: {

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
        BurstEngineWorker: BurstEngineWorkerError,
        BurstEngineWorkerPool: BurstEngineWorkerPoolError,
    },
}