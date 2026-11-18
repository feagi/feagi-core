//! A single Engine NPU. The NPU / Burst Loop and the engine run on the same thread, as theres
//! no point taking the performance hit the divide the two.

use crate::timing::std_impls::NPUTimings;
use crate::worker_commands::{NeuralProcessingUnitRequest, NeuralProcessingUnitResultant};
use burst_basis::connectome_editing::ConnectomeEditError;
use burst_basis::multi_engine::SingleEngineIndexing;
use burst_basis::visualization::BurstEngineVisualizationSupport;
use burst_basis::BurstEngine;
use connectome_basis::quantization_levels::FeagiIndexQuantization;
use feagi_basis::parallelism::feagi_channel::request_response_channel::dynamic::FeagiResponderAlloc;
use feagi_basis::parallelism::feagi_worker::worker_commands::{FeagiWorkerRequest, FeagiWorkerResponse};
use feagi_basis::parallelism::feagi_worker::worker_internals::feagi_worker_command_processor::FeagiWorkerCommandProcessor;
use feagi_basis::parallelism::feagi_worker::worker_internals::feagi_worker_command_router::FeagiWorkerCommandRouter;
use feagi_basis::target_frequency::TargetFrequency;
use std::marker::PhantomData;
use std::ops::ControlFlow;

/// The state of the single engine NPU that its commands act upon. Holds only settings; the burst
/// loop owns the schedule and reads these each tick
struct SingleEngineWorkerProcessor<FIQ, VIS, Engine>
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
    pub(crate) fn new(burst_engine: Engine, burst_frequency: TargetFrequency, visualization_frequency: TargetFrequency, paused: bool) -> Self {
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
