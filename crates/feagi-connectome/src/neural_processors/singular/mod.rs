//! A worker running a single burst engine at a target frequency

pub mod npu_loop;
pub mod commands;

use crate::neuron_models::init_dispatch::ConnectomeDispatch;
use commands::{NeuronProcessorSingleEngineCommandHandler, NeuronProcessorSingleEngineRequest, NeuronProcessorSingleEngineResponse};
use core::marker::PhantomData;
use feagi_basis::parallelism::feagi_channel::base_channels::base_channels::FeagiChannelBuilder;
use feagi_basis::parallelism::feagi_channel::base_channels::implementations::alloc_channels::FeagiChannelAllocBuilder;
use feagi_basis::parallelism::feagi_channel::request_response_channel::dynamic::FeagiRequestResponseChannelsAllocBackend;
use feagi_basis::parallelism::feagi_worker::feagi_worker::FeagiWorker;
use feagi_basis::parallelism::feagi_worker::feagi_worker_builder::{FeagiWorkerBuilder, WorkerStartError};
use feagi_basis::parallelism::feagi_worker::worker_internals::worker_spawner_implementation::impl_std::FeagiStdThreadSpawner;
use feagi_basis::target_frequency::TargetFrequency;
use feagi_connectome_neural::neural_basis::engine::{BurstEngine, BurstEngineIndex};
use feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
use feagi_connectome_neural::neural_basis::visualization::BurstVisualizationData;

/// How many commands can be in queue on this worker at one time
const COMMAND_QUEUE_LENGTH: usize = 8;

/// How many visualizations the visualization channel holds. One for the listener to read while the
/// engine writes the other
const VISUALIZATION_CHANNEL_SLOTS: usize = 2;

// TODO The vis traits may be more generic than the single engine

/// Sending half of the visualization channel, owned by the burst engine
pub type NeuronProcessorSingleEngineVisualizationTransmitter<FIQ> =
    <FeagiChannelAllocBuilder<BurstVisualizationData<FIQ>> as FeagiChannelBuilder<BurstVisualizationData<FIQ>>>::Transmitter;

/// Receiving half of the visualization channel, for whatever listens to the burst engine's
/// visualization
pub type NeuronProcessorSingleEngineVisualizationReceiver<FIQ> =
    <FeagiChannelAllocBuilder<BurstVisualizationData<FIQ>> as FeagiChannelBuilder<BurstVisualizationData<FIQ>>>::Receiver;


//endregion

//region Builder

/// Responsible for spawning a neuron processor running a single burst engine. The engine is
/// initialized on the worker's own thread, so only its init parameters need to be `Send`
pub struct NeuronProcessorSingleEngineWorkerBuilder<FIQ: FeagiIndexQuantization, Engine: BurstEngine<FIQ, ConnectomeDispatch, ()>> {
    /// The rate the burst loop starts ticking at
    pub initial_target_frequency: TargetFrequency,
    /// The rate bursts start sending visualization at
    pub initial_visualization_frequency: TargetFrequency,
    /// What the burst engine is initialized from
    pub engine_init_parameters: Engine::InitParameters,
}

// This is STD thread only for the time being
impl<FIQ, Engine> FeagiWorkerBuilder<FeagiStdThreadSpawner> for NeuronProcessorSingleEngineWorkerBuilder<FIQ, Engine>
where
    FIQ: FeagiIndexQuantization,
    Engine: BurstEngine<FIQ, ConnectomeDispatch, (), VisualizationTransmitter = NeuronProcessorSingleEngineVisualizationTransmitter<FIQ>>,
    Engine::InitParameters: 'static,
    Engine::InitError: core::error::Error + Send + 'static,
{
    type ChannelBackend = FeagiRequestResponseChannelsAllocBackend;
    /// The visualization transmitter from `spawn_related_channel_pairs`, handed to the engine
    type BuildInputs = NeuronProcessorSingleEngineVisualizationTransmitter<FIQ>;
    type BuildOutputs = ();
    /// The visualization channel. The receiver goes to whatever listens to visualization
    type RequiredChannelPairs = (
        NeuronProcessorSingleEngineVisualizationTransmitter<FIQ>,
        NeuronProcessorSingleEngineVisualizationReceiver<FIQ>,
    );
    type WorkerCommandHandler = NeuronProcessorSingleEngineCommandHandler<FIQ, Engine>;
    type WorkerStartError = NeuronProcessorSingleEngineWorkerBuilderError<Engine::InitError>;

    /// Blocks until the burst engine has initialized on the worker's thread, so an init failure is
    /// returned here rather than surfacing as a worker that exited

    fn build_worker(
        self,
        spawner: &FeagiStdThreadSpawner,
        visualization_transmitter: Self::BuildInputs,
    ) -> Result<
        (
            FeagiWorker<FeagiStdThreadSpawner, Self::ChannelBackend, NeuronProcessorSingleEngineRequest<FIQ>, NeuronProcessorSingleEngineResponse<FIQ>>,
            Self::BuildOutputs,
        ),
        WorkerStartError<Self::WorkerStartError>,
    > {
        let (cmd_requester, cmd_responder) = FeagiRequestResponseChannelsAllocBackend::create_channels(COMMAND_QUEUE_LENGTH);
        let (init_result_sender, init_result_receiver) = std::sync::mpsc::sync_channel(1);
        let Self { initial_target_frequency, initial_visualization_frequency, engine_init_parameters } = self;

        let handle = spawner.spawn_worker(move || async move {
            // Index will always be zero
            let engine = match Engine::init_burst_engine(
                engine_init_parameters,
                BurstEngineIndex(0),
                visualization_transmitter,
            ).await {
                Ok(engine) => {
                    // The builder only stops waiting by receiving, so this cannot fail
                    let _ = init_result_sender.send(Ok(()));
                    engine
                }
                Err(error) => {
                    let _ = init_result_sender.send(Err(error));
                    return;
                }
            };
            let handler = NeuronProcessorSingleEngineCommandHandler {
                target_frequency: initial_target_frequency,
                visualization_frequency: initial_visualization_frequency,
                paused: false,
                engine,
                _fiq: PhantomData,
            };
            npu_loop::run_neuron_processor_single_engine(cmd_responder, handler).await
        })?;

        match init_result_receiver.recv() {
            Ok(Ok(())) => Ok((FeagiWorker::new(cmd_requester, handle), ())),
            Ok(Err(error)) => Err(WorkerStartError::WorkerError(NeuronProcessorSingleEngineWorkerBuilderError::EngineInit(error))),
            // The thread exited (panicked) before reporting
            Err(_) => Err(WorkerStartError::WorkerError(NeuronProcessorSingleEngineWorkerBuilderError::WorkerExitedDuringInit)),
        }
    }

    fn spawn_related_channel_pairs(&self) -> Self::RequiredChannelPairs {
        FeagiChannelAllocBuilder::create_channels(VISUALIZATION_CHANNEL_SLOTS)
    }
}

/// Why the single engine neuron processor failed to start
#[derive(Debug, thiserror::Error)]
pub enum NeuronProcessorSingleEngineWorkerBuilderError<InitError: core::error::Error + 'static> {
    #[error("Burst engine failed to init: {0}")]
    EngineInit(#[source] InitError),
    #[error("Worker exited before the burst engine finished initializing")]
    WorkerExitedDuringInit,
}

//endregion


