/// How a worker is spawned within an API (std threads, embassy). NOT specific per worker but
/// rather backend implementation
pub trait WorkerSpawnerImplementation {
    // TODO a way to spawn threads for std or be assigned as task from embassy, possibly via subtype?

    type WorkerInternalHandle;
}

/// Backend failed to spawn a worker for some reason
#[derive(Debug, thiserror::Error)]
pub enum FeagiWorkerSpawnError {
    /// The backend has no room for another worker (embassy task pool full)
    #[error("No capacity for another worker!")]
    NoCapacity,
    /// The backend failed to start the worker (std thread creation failed)
    #[error("Backend failed to start the worker!")]
    BackendFailure,
}
