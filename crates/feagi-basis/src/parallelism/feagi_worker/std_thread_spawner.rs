//! Runs feagi workers on their own std threads.

use crate::parallelism::block_on_thread;
use crate::parallelism::feagi_worker::feagi_worker::FeagiWorkerSpawner;
use core::future::Future;

/// Spawns each worker on a new std thread and runs it there with `block_on_thread`
///
/// The body is a closure that builds the worker's future inside the new thread, so only the
/// closure needs to be `Send`, not the future
#[derive(Debug, Clone, Copy, Default)]
pub struct FeagiStdThreadSpawner;

impl<Body, BodyFuture> FeagiWorkerSpawner<Body> for FeagiStdThreadSpawner
where
    Body: FnOnce() -> BodyFuture + Send + 'static,
    BodyFuture: Future<Output=()>,
{
    type BackendHandle = std::thread::JoinHandle<()>;
    type SpawnError = std::io::Error;

    fn spawn_worker(&self, body: Body) -> Result<Self::BackendHandle, Self::SpawnError> {
        std::thread::Builder::new().spawn(move || block_on_thread(body()))
    }
}
