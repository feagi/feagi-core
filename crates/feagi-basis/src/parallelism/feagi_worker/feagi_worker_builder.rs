use crate::parallelism::feagi_channel::request_response_channel::FeagiRequestResponseChannelBackend;
use crate::parallelism::feagi_worker::worker_internals::worker_spawner_implementation::{FeagiWorkerSpawnError, WorkerSpawnerImplementation};
use crate::parallelism::feagi_worker::feagi_worker::FeagiWorker;
use crate::parallelism::feagi_worker::worker_internals::feagi_worker_command_processor::FeagiWorkerCommandProcessor;

/// Handles the construction of various 'FeagiWorker' types
pub trait FeagiWorkerBuilder<Spawner: WorkerSpawnerImplementation> {
    /// Which channel types (heap allocated, static) the built worker uses for its requests
    type ChannelBackend: FeagiRequestResponseChannelBackend;
    
    /// What the builder needs to build the worker, including any incoming request receiver
    /// channels from `spawn_required_channel_pairs`
    type BuildInputs;
    /// What the builder will output when building the worker (in addition to the worker)
    type BuildOutputs;
    /// Workers often have additional channels going in or out of themselves. This defines those
    type RequiredChannelPairs;
    
    /// Centralizes handling of commands for a worker
    type WorkerCommandHandler: FeagiWorkerCommandProcessor;

    /// Custom context for why a worker failed to start.
    type WorkerStartError: core::error::Error;

    /// Consume self to build the FeagiWorker
    fn build_worker(
        self,
        spawner: &Spawner,
        inputs: Self::BuildInputs,
    ) -> Result<
        (
            FeagiWorker<Spawner, Self::ChannelBackend, Self::CustomRequest, Self::CustomResponse>,
            Self::BuildOutputs,
        ),
        WorkerStartError<Self::WorkerStartError>,
    >;

    /// Create related channel pairings. Done first so different workers can be aligned before
    /// building them
    fn spawn_related_channel_pairs(&self) -> Self::RequiredChannelPairs;
}


#[derive(Debug, thiserror::Error)]
pub enum WorkerStartError<CustomError: core::error::Error> {
    #[error(transparent)]
    SpawnerError(FeagiWorkerSpawnError),
    #[error(transparent)]
    WorkerError(CustomError)
}

impl<CustomError: core::error::Error> From<FeagiWorkerSpawnError> for WorkerStartError<CustomError> {
    fn from(err: FeagiWorkerSpawnError) -> Self {
        WorkerStartError::SpawnerError(err)
    }
}
