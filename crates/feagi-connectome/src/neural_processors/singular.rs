//! A worker running a single burst engine at a target frequency

use feagi_basis::parallelism::feagi_channel::request_response_channel::dynamic::{FeagiRequestResponseChannelsAllocBackend, FeagiResponderAlloc};
use feagi_basis::parallelism::feagi_worker::feagi_worker::FeagiWorker;
use feagi_basis::parallelism::feagi_worker::feagi_worker_builder::{FeagiWorkerBuilder, WorkerStartError};
use feagi_basis::parallelism::feagi_worker::worker_commands::{FeagiWorkerRequest, FeagiWorkerResponse};
use feagi_basis::parallelism::feagi_worker::worker_internals::feagi_worker_command_processor::FeagiWorkerCommandProcessor;
use feagi_basis::parallelism::feagi_worker::worker_internals::feagi_worker_command_router::{FeagiWorkerPollingRouter, FeagiWorkerStep};
use feagi_basis::parallelism::feagi_worker::worker_internals::worker_spawner_implementation::impl_std::FeagiStdThreadSpawner;
use feagi_basis::target_frequency::TargetFrequency;
use std::time::Instant;
use feagi_connectome_neural::neural_basis::engine::burst_engine::BurstEngine;
use feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
use core::marker::PhantomData;
use feagi_basis::parallelism::feagi_channel::base_channels::base_channels::FeagiChannelBuilder;
use feagi_basis::parallelism::feagi_channel::base_channels::implementations::alloc_channels::FeagiChannelAllocBuilder;
use feagi_connectome_neural::neural_basis::visualization::BurstVisualizationData;

/// How many commands can be in queue on this worker at one time
const COMMAND_QUEUE_LENGTH: usize = 8;

/// How many visualizations the visualization channel holds. One for the listener to read while the
/// engine writes the other
const VISUALIZATION_CHANNEL_SLOTS: usize = 2;

/// Sending half of the visualization channel, owned by the burst engine
pub type NeuronProcessorSingleEngineVisualizationTransmitter<FIQ> =
    <FeagiChannelAllocBuilder<BurstVisualizationData<FIQ>> as FeagiChannelBuilder<BurstVisualizationData<FIQ>>>::Transmitter;

/// Receiving half of the visualization channel, for whatever listens to the burst engine's
/// visualization
pub type NeuronProcessorSingleEngineVisualizationReceiver<FIQ> =
    <FeagiChannelAllocBuilder<BurstVisualizationData<FIQ>> as FeagiChannelBuilder<BurstVisualizationData<FIQ>>>::Receiver;

//region Commands

/// Custom requests to the single engine neuron processor
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NeuronProcessorSingleEngineRequest {
    /// Set the rate the burst loop ticks at. Applies from the next tick
    TargetFrequency(TargetFrequency),
    /// Set the rate bursts send visualization at. Bursts never run faster than the target
    /// frequency, so a visualization frequency above it means every burst sends visualization
    VisualizationFrequency(TargetFrequency),
}

/// Custom responses from the single engine neuron processor
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NeuronProcessorSingleEngineResponse {
    /// The target frequency was applied
    TargetFrequencyUpdated,
    /// The visualization frequency was applied
    VisualizationFrequencyUpdated,
}

/// The state of the single engine neuron processor, which its commands act upon
pub struct NeuronProcessorSingleEngineCommandHandler<FIQ: FeagiIndexQuantization, Engine: BurstEngine<FIQ, ()>> {
    target_frequency: TargetFrequency,
    visualization_frequency: TargetFrequency,
    engine: Engine,
    _fiq: PhantomData<fn() -> FIQ>,
}

impl<FIQ: FeagiIndexQuantization, Engine: BurstEngine<FIQ, ()>> FeagiWorkerCommandProcessor
for NeuronProcessorSingleEngineCommandHandler<FIQ, Engine> {
    type CustomRequest = NeuronProcessorSingleEngineRequest;
    type CustomResponse = NeuronProcessorSingleEngineResponse;

    async fn on_custom(&mut self, request: Self::CustomRequest) -> Self::CustomResponse {
        match request {
            NeuronProcessorSingleEngineRequest::TargetFrequency(target_frequency) => {
                self.target_frequency = target_frequency;
                NeuronProcessorSingleEngineResponse::TargetFrequencyUpdated
            }
            NeuronProcessorSingleEngineRequest::VisualizationFrequency(visualization_frequency) => {
                self.visualization_frequency = visualization_frequency;
                NeuronProcessorSingleEngineResponse::VisualizationFrequencyUpdated
            }
        }
    }
}

//endregion

//region Builder

/// Responsible for spawning a neuron processor running a single burst engine. The engine is
/// initialized on the worker's own thread, so only its init parameters need to be `Send`
pub struct NeuronProcessorSingleEngineWorkerBuilder<FIQ: FeagiIndexQuantization, Engine: BurstEngine<FIQ, ()>> {
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
    Engine: BurstEngine<FIQ, (), VisualizationTransmitter = NeuronProcessorSingleEngineVisualizationTransmitter<FIQ>>,
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
            FeagiWorker<FeagiStdThreadSpawner, Self::ChannelBackend, NeuronProcessorSingleEngineRequest, NeuronProcessorSingleEngineResponse>,
            Self::BuildOutputs,
        ),
        WorkerStartError<Self::WorkerStartError>,
    > {
        let (cmd_requester, cmd_responder) = FeagiRequestResponseChannelsAllocBackend::create_channels(COMMAND_QUEUE_LENGTH);
        let (init_result_sender, init_result_receiver) = std::sync::mpsc::sync_channel(1);
        let Self { initial_target_frequency, initial_visualization_frequency, engine_init_parameters } = self;

        let handle = spawner.spawn_worker(move || async move {
            let engine = match Engine::init_burst_engine(engine_init_parameters, visualization_transmitter).await {
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
                engine,
                _fiq: PhantomData,
            };
            run_neuron_processor_single_engine(cmd_responder, handler).await
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

//region Loop

/// The burst loop. Each tick: schedules the next tick, handles every queued command, runs the
/// burst (sending visualization if it is due), then waits until the scheduled tick
async fn run_neuron_processor_single_engine<FIQ: FeagiIndexQuantization, Engine: BurstEngine<FIQ, ()>>(
    responder: FeagiResponderAlloc<
        FeagiWorkerRequest<NeuronProcessorSingleEngineRequest>,
        FeagiWorkerResponse<NeuronProcessorSingleEngineResponse>,
    >,
    handler: NeuronProcessorSingleEngineCommandHandler<FIQ, Engine>,
) {
    let mut commands = FeagiWorkerPollingRouter::new(handler, responder);
    let mut tick_time = Instant::now();
    let mut visualization_time = Instant::now();

    loop {
        // Scheduled from the previous tick so the rate does not drift. If the previous tick ran
        // over, schedule from now instead of bursting to catch up
        let tick_duration = commands.processor_mut().target_frequency.duration_between_ticks();
        tick_time = (tick_time + tick_duration).max(Instant::now());

        loop {
            match commands.try_handle_next().await {
                FeagiWorkerStep::Handled => continue,
                FeagiWorkerStep::Idle => break,
                FeagiWorkerStep::Stopped | FeagiWorkerStep::Disconnected => return,
            }
        }

        // Visualization is scheduled the same way as ticks, but only checked once per burst
        let now = Instant::now();
        let send_visualization = now >= visualization_time;
        if send_visualization {
            let visualization_duration = commands.processor_mut().visualization_frequency.duration_between_ticks();
            visualization_time = (visualization_time + visualization_duration).max(now);
        }

        commands.processor_mut().engine.run_burst(send_visualization).await;

        wait_until(tick_time).await;
    }
}

/// Waits until `deadline`. The worker has its own thread, so this blocks that thread
async fn wait_until(deadline: Instant) {
    let now = Instant::now();
    if deadline > now {
        std::thread::sleep(deadline - now);
    }
}

//endregion
