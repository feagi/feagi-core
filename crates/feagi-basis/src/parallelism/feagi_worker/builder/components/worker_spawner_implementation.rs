/// How a worker is spawned within an API (std threads, embassy). NOT specific per worker but
/// rather backend implementation
pub trait WorkerSpawnerImplementation {
    // TODO a way to spawn threads for std or be assigned as task from embassy, possibly via subtype?
    // The internal method used by the spawner to hold onto this 
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

#[cfg(feature = "std")]
pub mod impl_std {
    use super::{FeagiWorkerSpawnError, WorkerSpawnerImplementation};

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
            BodyFuture: core::future::Future<Output=()>,
        {
            std::thread::Builder::new()
                .spawn(move || crate::parallelism::block_on_thread(body()))
                .map_err(|_| FeagiWorkerSpawnError::BackendFailure)
        }
    }
}