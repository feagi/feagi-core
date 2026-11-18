//! A single Engine NPU. The NPU / Burst Loop and the engine run on the same thread, as theres
//! no point taking the performance hit the divide the two.

use crate::timing::std_impls::NPUTimings;
use crate::worker_commands::{NeuralProcessingUnitRequest, NeuralProcessingUnitResultant};
use burst_basis::connectome_editing::ConnectomeEditError;
use burst_basis::multi_engine::{BurstEngineIndex, SingleEngineIndexing};
use burst_basis::visualization::{AllocVisualizationReceiver, AllocVisualizationTransmitter, BurstEngineVisualizationSupport};
use burst_basis::BurstEngine;
use connectome_basis::quantization_levels::FeagiIndexQuantization;
use feagi_basis::parallelism::feagi_channel::request_response_channel::dynamic::{FeagiRequestResponseChannelsAllocBackend, FeagiResponderAlloc};
use feagi_basis::parallelism::feagi_worker::worker_commands::{FeagiWorkerRequest, FeagiWorkerResponse};
use feagi_basis::parallelism::feagi_worker::worker_internals::feagi_worker_command_processor::FeagiWorkerCommandProcessor;
use feagi_basis::parallelism::feagi_worker::worker_internals::feagi_worker_command_router::FeagiWorkerCommandRouter;
use feagi_basis::target_frequency::TargetFrequency;
use std::marker::PhantomData;
use std::ops::ControlFlow;
use feagi_basis::parallelism::feagi_channel::base_channels::implementations::alloc_channels::FeagiChannelAllocBuilder;
use feagi_basis::parallelism::feagi_worker::feagi_worker::FeagiWorker;
use feagi_basis::parallelism::feagi_worker::feagi_worker_builder::{FeagiWorkerBuilder, WorkerStartError};
use feagi_basis::parallelism::feagi_worker::worker_internals::worker_spawner_implementation::impl_std::FeagiStdThreadSpawner;

/// How many commands can be in queue on this worker at one time
const COMMAND_QUEUE_LENGTH: usize = 8;

/// How many visualizations the visualization channel holds. One for the listener to read while the
/// engine writes the other
const VISUALIZATION_CHANNEL_SLOTS: usize = 2;

/// Responsible for spawning a neuron processor running a single burst engine. The engine is
/// initialized on the worker's own thread, so only its init parameters need to be `Send`
pub struct NeuronProcessorSingleEngineWorkerBuilder<
    FIQ: FeagiIndexQuantization,
    VIS: BurstEngineVisualizationSupport,
    Engine: BurstEngine<FIQ, VIS>
> {
    /// The rate the burst loop starts ticking at
    pub initial_target_frequency: TargetFrequency,
    /// The rate bursts start sending visualization at
    pub initial_visualization_frequency: TargetFrequency,
    /// What the burst engine is initialized from
    pub engine_init_parameters: Engine::InitParameters,
}

// This is STD thread only for the time being
impl<FIQ, VIS, Engine> FeagiWorkerBuilder<FeagiStdThreadSpawner> for NeuronProcessorSingleEngineWorkerBuilder<FIQ, VIS, Engine>
where
    FIQ: FeagiIndexQuantization,
    VIS: BurstEngineVisualizationSupport,
    Engine: BurstEngine<FIQ, VIS>
{
    type ChannelBackend = FeagiRequestResponseChannelsAllocBackend;
    /// The visualization transmitter from `spawn_related_channel_pairs`, handed to the engine
    type BuildInputs = VIS::VisualizationTransmitter;
    type BuildOutputs = ();
    /// The visualization channel. The receiver goes to whatever listens to visualization
    type RequiredChannelPairs = (
        AllocVisualizationTransmitter<FIQ>,
        AllocVisualizationReceiver<FIQ>,
    );
    type WorkerCommandHandler = SingleEngineWorkerProcessor<FIQ, VIS, Engine>;
    type WorkerStartError = NeuronProcessorSingleEngineWorkerBuilderError<Engine::InitError>;


    fn build_worker(
        self,
        spawner: &FeagiStdThreadSpawner,
        inputs: Self::BuildInputs
    ) -> Result<
        (
            FeagiWorker<FeagiStdThreadSpawner, Self::ChannelBackend,
                <Self::WorkerCommandHandler as FeagiWorkerCommandProcessor>::CustomRequest,
                <Self::WorkerCommandHandler as FeagiWorkerCommandProcessor>::CustomResponse>,
            Self::BuildOutputs
        ),
        WorkerStartError<Self::WorkerStartError>>
    {
        let (cmd_requester, cmd_responder) = FeagiRequestResponseChannelsAllocBackend::create_channels(COMMAND_QUEUE_LENGTH);
        let (init_result_sender, init_result_receiver) = std::sync::mpsc::sync_channel(1);
        let Self {
            initial_target_frequency,
            initial_visualization_frequency,
            engine_init_parameters,
        } = self;

        let handle = spawner.spawn_worker(move || async move {
            // Index will always be zero
            let engine = match Engine::init_burst_engine(engine_init_parameters, BurstEngineIndex(0), inputs).await {
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
            let handler = SingleEngineWorkerProcessor::new(
                engine,
                initial_target_frequency,
                initial_visualization_frequency,
                false);
            burst_loop(cmd_responder, handler).await
        })?;

        match init_result_receiver.recv() {
            Ok(Ok(())) => Ok((FeagiWorker::new(cmd_requester, handle), ())),
            Ok(Err(error)) => Err(WorkerStartError::WorkerError(NeuronProcessorSingleEngineWorkerBuilderError::EngineInit(
                error,
            ))),
            // The thread exited (panicked) before reporting
            Err(_) => Err(WorkerStartError::WorkerError(
                NeuronProcessorSingleEngineWorkerBuilderError::WorkerExitedDuringInit,
            )),
        }
    }

    fn spawn_related_channel_pairs(&self) -> Self::RequiredChannelPairs {
        FeagiChannelAllocBuilder::create_channels(VISUALIZATION_CHANNEL_SLOTS)
    }
}




//region Processor

/// The state of the single engine NPU that its commands act upon. Holds only settings; the burst
/// loop owns the schedule and reads these each tick
pub struct SingleEngineWorkerProcessor<FIQ, VIS, Engine>
where
    FIQ: FeagiIndexQuantization,
    VIS: BurstEngineVisualizationSupport,
    Engine: BurstEngine<FIQ, VIS>,
{
    burst_frequency: TargetFrequency,
    visualization_frequency: TargetFrequency,
    /// While paused, no bursts run and the worker only waits for commands
    paused: bool,
    burst_engine: Engine,
    _types: PhantomData<fn() -> (FIQ, VIS)>,
}

impl<FIQ, VIS, Engine> SingleEngineWorkerProcessor<FIQ, VIS, Engine>
where
    FIQ: FeagiIndexQuantization,
    VIS: BurstEngineVisualizationSupport,
    Engine: BurstEngine<FIQ, VIS>,
{
    fn new(burst_engine: Engine, burst_frequency: TargetFrequency, visualization_frequency: TargetFrequency, paused: bool) -> Self {
        Self {
            burst_frequency,
            visualization_frequency,
            paused,
            burst_engine,
            _types: PhantomData,
        }
    }
}

impl<FIQ, VIS, Engine> FeagiWorkerCommandProcessor for SingleEngineWorkerProcessor<FIQ, VIS, Engine>
where
    FIQ: FeagiIndexQuantization,
    VIS: BurstEngineVisualizationSupport,
    Engine: BurstEngine<FIQ, VIS>,
{
    type CustomRequest = NeuralProcessingUnitRequest<FIQ, SingleEngineIndexing>;
    type CustomResponse = NeuralProcessingUnitResultant<FIQ, SingleEngineIndexing>;

    /// The burst loop reads both rates each tick, so a rate set here applies from the next tick
    async fn on_custom(&mut self, request: Self::CustomRequest) -> Self::CustomResponse {
        match request {
            NeuralProcessingUnitRequest::BurstRate(burst_rate) => {
                self.burst_frequency = burst_rate;
                NeuralProcessingUnitResultant::BurstRate
            }
            NeuralProcessingUnitRequest::VisualizationRate(vis_rate) => {
                self.visualization_frequency = vis_rate;
                NeuralProcessingUnitResultant::VisualizationRate
            }
            NeuralProcessingUnitRequest::Pause => {
                self.paused = true;
                NeuralProcessingUnitResultant::Paused
            }
            NeuralProcessingUnitRequest::Resume => {
                self.paused = false;
                NeuralProcessingUnitResultant::Resumed
            }
            NeuralProcessingUnitRequest::ConnectomeEdit(i, edit) => {
                self.paused = true;
                // NOTE: System will pause when edits are submitted, and must be unpaused after

                // index is irrelevant, just return it
                NeuralProcessingUnitResultant::ConnectomeEdit(i, self.burst_engine.handle_connectome_edit_enum(edit).await)
            }
            NeuralProcessingUnitRequest::ConnectomeEdits(edits) => {
                self.paused = true; // NOTE: System will pause when edits are submitted, and must be unpaused after
                let mut successfuls = vec![];
                for (engine_index, edit) in edits {
                    let result = self.burst_engine.handle_connectome_edit_enum(edit).await;
                    match result {
                        Ok(r) => {
                            successfuls.push(r);
                        }
                        Err(e) => {
                            return NeuralProcessingUnitResultant::ConnectomeEdits(
                                Err(ConnectomeEditError::GeneralEngineFailure("TODO fix me", false)), // TODO actual error parsing
                            );
                        }
                    }
                }
                return NeuralProcessingUnitResultant::ConnectomeEdits(Ok(successfuls));
            }
        }
    }
}

//endregion

/// Why the single engine neuron processor failed to start
#[derive(Debug, thiserror::Error)]
pub enum NeuronProcessorSingleEngineWorkerBuilderError<InitError: core::error::Error + 'static> {
    #[error("Burst engine failed to init: {0}")]
    EngineInit(#[source] InitError),
    #[error("Worker exited before the burst engine finished initializing")]
    WorkerExitedDuringInit,
}

/// The burst loop of a single engine NPU. Since theres only 1 engine, we just reuse the thread
/// of the NPU anyways. STD is required for this npu, so can be assumed for all engines.
async fn burst_loop<FIQ, VIS, Engine>(
    responder: FeagiResponderAlloc<
        FeagiWorkerRequest<NeuralProcessingUnitRequest<FIQ, SingleEngineIndexing>>,
        FeagiWorkerResponse<NeuralProcessingUnitResultant<FIQ, SingleEngineIndexing>>,
    >,
    handler: SingleEngineWorkerProcessor<FIQ, VIS, Engine>,
) where
    FIQ: FeagiIndexQuantization,
    VIS: BurstEngineVisualizationSupport,
    Engine: BurstEngine<FIQ, VIS>,
{
    let mut command_router = FeagiWorkerCommandRouter::new(handler, responder);
    let mut timings = NPUTimings::new();

    loop {
        // Scheduled from the previous tick so the rate does not drift
        let burst_frequency = command_router.processor_mut().burst_frequency;
        timings.schedule_next_burst(burst_frequency);

        loop {
            match command_router.try_handle_next().await {
                ControlFlow::Continue(true) => continue, // still more commands in queue, process immediately
                ControlFlow::Continue(false) => break,   // no more commands in queue
                ControlFlow::Break(_) => return,         // channel killed
            }
        }

        // While paused, nothing runs on the timer. The worker waits for commands and handles each
        // one as it arrives, without spending time between them
        if command_router.processor_mut().paused {
            while command_router.processor_mut().paused {
                if command_router.await_handle_next().await.is_break() {
                    return;
                }
            }
            // Resumed. Burst now, and schedule the next tick from now rather than catching up
            let burst_frequency = command_router.processor_mut().burst_frequency;
            timings.schedule_next_burst_from_now(burst_frequency);
        }

        let visualization_frequency = command_router.processor_mut().visualization_frequency;
        let send_visualization = timings.take_visualization_due(visualization_frequency);

        command_router.processor_mut().burst_engine.run_burst(send_visualization).await;

        timings.wait_for_next_burst().await;
    }
}
