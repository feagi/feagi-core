use crate::parallelism::feagi_worker::feagi_worker::FeagiWorker;
use crate::parallelism::feagi_worker::feagi_worker_channel::FeagiWorkerChannels;
use crate::parallelism::feagi_worker::worker_spawner_implementation::WorkerSpawnerImplementation;

/// Handles the construction of various 'FeagiWorker' types
pub trait FeagiWorkerBuilder<Spawner: WorkerSpawnerImplementation> {
    /// Which channel types (heap allocated, static) the built worker uses for its requests
    type Channels: FeagiWorkerChannels;
    /// What the builder needs to build the worker, including any incoming request receiver
    /// channels from `spawn_required_channel_pairs`
    type BuildInputs;
    /// What the builder will output when building the worker (in addition to the worker)
    type BuildOutputs;
    /// Workers often have additional channels going in or out of themselves. This defines those
    type RequiredChannelPairs;
    /// What type of custom request going in is supported
    type CustomRequest;
    /// What type of custom response coming out is supported
    type CustomResponse;

    type BuildError: core::error::Error;

    /// Consume self to build the FeagiWorker
    fn build_worker(
        self,
        spawner: &Spawner,
        inputs: Self::BuildInputs,
    ) -> Result<
        (
            FeagiWorker<Spawner, Self::Channels, Self::CustomRequest, Self::CustomResponse>,
            Self::BuildOutputs,
        ),
        Self::BuildError,
    >;

    /// Create needed channel pairings. Done first so different workers can be aligned before
    /// building them
    fn spawn_required_channel_pairs(&self) -> Self::RequiredChannelPairs;
}