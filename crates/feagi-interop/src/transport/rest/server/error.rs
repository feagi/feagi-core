use feagi_basis::parallelism::feagi_channel::channel_error::FeagiRequestError;
use feagi_basis::parallelism::feagi_worker::worker_spawner_implementation::FeagiWorkerSpawnError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FeagiRestServerError {
    #[error("Server failed to bind to socket!")] // TODO socket?
    FailedToBindToSocket,
    #[error("Server failed to configure its socket!")]
    FailedToConfigureSocket,
    #[error("Server failed to parse data!")]
    ServerIncomingDataParseFailure,
    #[error("Server worker failed to start: {0}")]
    FailedToStartWorker(#[from] FeagiWorkerSpawnError),
    #[error("Server worker failed to acknowledge stopping: {0}")]
    FailedToStopWorker(#[from] FeagiRequestError),
    #[error("Server worker panicked!")]
    WorkerPanicked,
}
