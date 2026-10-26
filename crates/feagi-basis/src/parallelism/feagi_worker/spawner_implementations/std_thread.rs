use crate::parallelism::feagi_worker::worker_spawner_implementation::{FeagiWorkerSpawnError, WorkerSpawnerImplementation};

//region Worker Spawner Implementation

/// Spawns each worker on a new std thread and runs it there with `block_on_thread`
#[derive(Debug, Clone, Copy, Default)]
pub struct FeagiStdThreadSpawner;

impl WorkerSpawnerImplementation for FeagiStdThreadSpawner {
    type WorkerInternalHandle = std::thread::JoinHandle<()>;
}

impl FeagiStdThreadSpawner {
    /// Starts `body` on a new thread and blocks that thread on the future it returns
    ///
    /// The body is a closure that builds the worker's future inside the new thread, so only the
    /// closure needs to be `Send`, not the future
    ///
    /// Errors with `BackendFailure` if the OS could not create the thread
    pub fn spawn_worker<Body, BodyFuture>(&self, body: Body) -> Result<std::thread::JoinHandle<()>, FeagiWorkerSpawnError>
    where
        Body: FnOnce() -> BodyFuture + Send + 'static,
        BodyFuture: core::future::Future<Output = ()>,
    {
        std::thread::Builder::new()
            .spawn(move || crate::parallelism::block_on_thread(body()))
            .map_err(|_| FeagiWorkerSpawnError::BackendFailure)
    }
}

//endregion
