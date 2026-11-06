//! A worker running a single burst engine at a target frequency

use core::marker::PhantomData;
use feagi_basis::parallelism::feagi_channel::base_channels::base_channels::FeagiChannelBuilder;
use feagi_basis::parallelism::feagi_channel::base_channels::implementations::alloc_channels::FeagiChannelAllocBuilder;
use feagi_basis::parallelism::feagi_channel::request_response_channel::dynamic::{FeagiRequestResponseChannelsAllocBackend, FeagiResponderAlloc};
use feagi_basis::parallelism::feagi_worker::feagi_worker::FeagiWorker;
use feagi_basis::parallelism::feagi_worker::feagi_worker_builder::{FeagiWorkerBuilder, WorkerStartError};
use feagi_basis::parallelism::feagi_worker::worker_commands::{FeagiWorkerRequest, FeagiWorkerResponse};
use feagi_basis::parallelism::feagi_worker::worker_internals::feagi_worker_command_processor::FeagiWorkerCommandProcessor;
use feagi_basis::parallelism::feagi_worker::worker_internals::feagi_worker_command_router::{FeagiWorkerPollingRouter, FeagiWorkerStep};
use feagi_basis::parallelism::feagi_worker::worker_internals::worker_spawner_implementation::impl_std::FeagiStdThreadSpawner;
use feagi_basis::target_frequency::TargetFrequency;
use crate::neuron_models::init_dispatch::ConnectomeDispatch;
use feagi_connectome_neural::neural_basis::connectome_editing::{ConnectomeEditResultantEnum, ConnectomeEditStepEnum};
use feagi_connectome_neural::neural_basis::engine::engine_change_command::EngineChangeCommandOtherError;
use feagi_connectome_neural::neural_basis::engine::{BurstEngine, BurstEngineIndex};
use feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
use feagi_connectome_neural::neural_basis::visualization::BurstVisualizationData;
use std::time::Instant;

/// How many commands can be in queue on this worker at one time
const COMMAND_QUEUE_LENGTH: usize = 8;

/// How long the worker sleeps between checks for a command while paused. Bounds how late a command
/// is answered while paused
const PAUSED_POLL_INTERVAL: core::time::Duration = core::time::Duration::from_millis(1);

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
pub enum NeuronProcessorSingleEngineRequest<FIQ: FeagiIndexQuantization> {
    /// Set the rate the burst loop ticks at. Applies from the next tick, if the engine accepts it
    BurstRate(TargetFrequency),
    /// Set the rate bursts send visualization at, if the engine accepts it. Bursts never run faster
    /// than the burst rate, so a visualization rate above it means every burst sends visualization
    VisualizationRate(TargetFrequency),
    /// Apply these connectome edit steps in order, all between the same two bursts. Stops at the
    /// first step that fails
    ConnectomeEdits(Vec<ConnectomeEditStepEnum<FIQ, ConnectomeDispatch>>),
    /// Stop bursting until resumed. While paused the worker only waits for and handles commands,
    /// so its caller can edit the connectome and update its own state before bursting continues
    Pause,
    /// Continue bursting after a pause, from now rather than catching up on the bursts missed
    Resume,
}

/// Custom responses from the single engine neuron processor, one per kind of request
pub enum NeuronProcessorSingleEngineResponse<FIQ: FeagiIndexQuantization> {
    /// Whether the engine accepted the burst rate. If it did not, the rate is unchanged
    BurstRate(Result<(), EngineChangeCommandOtherError>),
    /// Whether the engine accepted the visualization rate. If it did not, the rate is unchanged
    VisualizationRate(Result<(), EngineChangeCommandOtherError>),
    /// What each edit step that was attempted returned, in order. If the last one failed, the
    /// steps after it were not attempted
    ConnectomeEdits(Vec<Result<ConnectomeEditResultantEnum<FIQ>, EngineChangeCommandOtherError>>),
    /// Bursting is paused. Also the reply if it already was
    Paused,
    /// Bursting has resumed. Also the reply if it was not paused
    Resumed,
}

/// The state of the single engine neuron processor, which its commands act upon
pub struct NeuronProcessorSingleEngineCommandHandler<FIQ: FeagiIndexQuantization, Engine: BurstEngine<FIQ, ConnectomeDispatch, ()>> {
    target_frequency: TargetFrequency,
    visualization_frequency: TargetFrequency,
    /// While paused, no bursts run and the worker only waits for commands
    paused: bool,
    engine: Engine,
    _fiq: PhantomData<fn() -> FIQ>,
}

impl<FIQ: FeagiIndexQuantization, Engine: BurstEngine<FIQ, ConnectomeDispatch, ()>> NeuronProcessorSingleEngineCommandHandler<FIQ, Engine> {
    /// Relays each edit step to the engine in order, stopping at the first that fails, whether or
    /// not it corrupted the connectome. Deciding what to do about corruption is left to the caller
    async fn apply_connectome_edits(
        &mut self,
        steps: Vec<ConnectomeEditStepEnum<FIQ, ConnectomeDispatch>>,
    ) -> Vec<Result<ConnectomeEditResultantEnum<FIQ>, EngineChangeCommandOtherError>> {
        let mut results = Vec::with_capacity(steps.len());
        for step in steps {
            let result = self.engine.process_connectome_edit(step).await;
            let failed = result.as_ref().map_or(true, ConnectomeEditResultantEnum::is_err);
            results.push(result);
            if failed {
                break;
            }
        }
        results
    }
}

impl<FIQ: FeagiIndexQuantization, Engine: BurstEngine<FIQ, ConnectomeDispatch, ()>> FeagiWorkerCommandProcessor
for NeuronProcessorSingleEngineCommandHandler<FIQ, Engine> {
    type CustomRequest = NeuronProcessorSingleEngineRequest<FIQ>;
    type CustomResponse = NeuronProcessorSingleEngineResponse<FIQ>;

    /// The burst loop is timed here, so a rate the engine accepts also applies from the next tick
    async fn on_custom(&mut self, request: Self::CustomRequest) -> Self::CustomResponse {
        match request {
            NeuronProcessorSingleEngineRequest::BurstRate(burst_rate) => {
                let result = self.engine.set_burst_rate(burst_rate).await;
                if result.is_ok() {
                    self.target_frequency = burst_rate;
                }
                NeuronProcessorSingleEngineResponse::BurstRate(result)
            }
            NeuronProcessorSingleEngineRequest::VisualizationRate(visualization_rate) => {
                let result = self.engine.set_visualization_rate(visualization_rate).await;
                if result.is_ok() {
                    self.visualization_frequency = visualization_rate;
                }
                NeuronProcessorSingleEngineResponse::VisualizationRate(result)
            }
            NeuronProcessorSingleEngineRequest::ConnectomeEdits(steps) => {
                NeuronProcessorSingleEngineResponse::ConnectomeEdits(self.apply_connectome_edits(steps).await)
            }
            NeuronProcessorSingleEngineRequest::Pause => {
                self.paused = true;
                NeuronProcessorSingleEngineResponse::Paused
            }
            NeuronProcessorSingleEngineRequest::Resume => {
                self.paused = false;
                NeuronProcessorSingleEngineResponse::Resumed
            }
        }
    }
}

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
                initial_target_frequency,
                initial_visualization_frequency,
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
/// burst (sending visualization if it is due), then waits until the scheduled tick. If a command
/// paused it, it instead waits on and handles commands until resumed, then bursts
async fn run_neuron_processor_single_engine<FIQ: FeagiIndexQuantization, Engine: BurstEngine<FIQ, ConnectomeDispatch, ()>>(
    responder: FeagiResponderAlloc<
        FeagiWorkerRequest<NeuronProcessorSingleEngineRequest<FIQ>>,
        FeagiWorkerResponse<NeuronProcessorSingleEngineResponse<FIQ>>,
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

        // While paused, nothing runs on the timer. Only commands are handled, as they arrive
        if commands.processor_mut().paused {
            while commands.processor_mut().paused {
                match commands.try_handle_next().await {
                    FeagiWorkerStep::Handled => {}
                    FeagiWorkerStep::Idle => std::thread::sleep(PAUSED_POLL_INTERVAL),
                    FeagiWorkerStep::Stopped | FeagiWorkerStep::Disconnected => return,
                }
            }
            // Resumed. Burst now, and schedule the next tick from now rather than catching up
            let tick_duration = commands.processor_mut().target_frequency.duration_between_ticks();
            tick_time = Instant::now() + tick_duration;
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
