
use crate::parallelism::feagi_worker::builder::components::worker_spawner_implementation::FeagiWorkerSpawnError;


/// Backend failed to spawn a worker for some reason
#[derive(Debug, thiserror::Error)]
pub enum FeagiWorkerBuilderError<CustomError: core::error::Error> {
    /// The backend failed for some reason to init the worker
    #[error(transparent)]
    BackendInitError(FeagiWorkerSpawnError),
    /// Worker failed to spawn
    #[error(transparent)]
    WorkerFailedToSpawn(CustomError)
}

impl<CustomError: core::error::Error> From<FeagiWorkerSpawnError> for FeagiWorkerBuilderError<CustomError> {
    fn from(value: FeagiWorkerSpawnError) -> Self {
        FeagiWorkerBuilderError::BackendInitError(value)
    }
}
