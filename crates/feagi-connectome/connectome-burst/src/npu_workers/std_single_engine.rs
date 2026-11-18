//! A single Engine NPU. The NPU / Burst Loop and the engine run on the same thread, as theres
//! no point taking the performance hit the divide the two.

use burst_basis::BurstEngine;
use burst_basis::connectome_editing::ConnectomeEditError;
use burst_basis::multi_engine::SingleEngineIndexing;
use burst_basis::visualization::BurstEngineVisualizationSupport;
use connectome_basis::quantization_levels::FeagiIndexQuantization;
use feagi_basis::parallelism::feagi_channel::request_response_channel::dynamic::FeagiResponderAlloc;
use feagi_basis::parallelism::feagi_worker::worker_commands::{FeagiWorkerRequest, FeagiWorkerResponse};
use feagi_basis::parallelism::feagi_worker::worker_internals::feagi_worker_command_processor::FeagiWorkerCommandProcessor;
use feagi_basis::parallelism::feagi_worker::worker_internals::feagi_worker_command_router::FeagiWorkerCommandRouter;
use crate::timing::std_impls::NPUTimings;
use crate::worker_commands::{NeuralProcessingUnitRequest, NeuralProcessingUnitResultant};


struct SingleEngineWorkerProcessor<FIQ, VIS, Engine>
where
    FIQ: FeagiIndexQuantization,
    VIS: BurstEngineVisualizationSupport,
    Engine: BurstEngine<FIQ, VIS>
{
    timings: NPUTimings,
    burst_engine: Engine
}





impl<FIQ, VIS, Engine> FeagiWorkerCommandProcessor for SingleEngineWorkerProcessor<FIQ, VIS, Engine>
where
    FIQ: FeagiIndexQuantization,
    VIS: BurstEngineVisualizationSupport,
    Engine: BurstEngine<FIQ, VIS>
{
    type CustomRequest = NeuralProcessingUnitRequest<FIQ, SingleEngineIndexing>;
    type CustomResponse = NeuralProcessingUnitResultant<FIQ, SingleEngineIndexing>;

    async fn on_custom(&mut self, request: Self::CustomRequest) -> Self::CustomResponse {
        match request {
            NeuralProcessingUnitRequest::BurstRate(burst_rate) => {
                self.timings.set_burst_frequency(burst_rate);
                NeuralProcessingUnitResultant::BurstRate
            }
            NeuralProcessingUnitRequest::VisualizationRate(vis_rate) => {
                self.timings.set_visualization_frequency(vis_rate);
                NeuralProcessingUnitResultant::VisualizationRate
            }
            NeuralProcessingUnitRequest::Pause => {
                self.timings.set_paused(true);
                NeuralProcessingUnitResultant::Paused
            }
            NeuralProcessingUnitRequest::Resume => {
                self.timings.set_paused(false);
                NeuralProcessingUnitResultant::Resumed
            }
            NeuralProcessingUnitRequest::ConnectomeEdit(i, edit) => {
                // index is irrelevant
                NeuralProcessingUnitResultant::ConnectomeEdit(
                    i,
                    self.burst_engine.handle_connectome_edit_enum(edit).await
                )
            }
            NeuralProcessingUnitRequest::ConnectomeEdits(edits) => {
                let mut successfuls = vec![];
                for (engine_index, edit) in edits {
                    let result = self.burst_engine.handle_connectome_edit_enum(edit).await;
                        match result {
                        Ok(r) => {
                            successfuls.push(r);
                        },
                        Err(e) => {
                            return NeuralProcessingUnitResultant::ConnectomeEdits(
                                Err(ConnectomeEditError::GeneralEngineFailure("TODO fix me", false)) // TODO actual error parsing
                            )
                        }
                    }
                };
                return NeuralProcessingUnitResultant::ConnectomeEdits(Ok(successfuls))
            }
        }
    }
}





/// The burst loop of a single engine NPU. Since theres only 1 engine, we just reuse the thread
/// of the NPU anyways. STD is required for this npu, so can be assumed for all engines
async fn burst_loop<FIQ, VIS, Engine>(
    responder: FeagiResponderAlloc<
        FeagiWorkerRequest<NeuralProcessingUnitRequest<FIQ, SingleEngineIndexing>>,
        FeagiWorkerResponse<NeuralProcessingUnitResultant<FIQ, SingleEngineIndexing>>,
    >,
    handler: SingleEngineWorkerProcessor<FIQ, VIS, Engine>,
)
where
    FIQ: FeagiIndexQuantization,
    VIS: BurstEngineVisualizationSupport,
    Engine: BurstEngine<FIQ, VIS>
{
    let mut command_router = FeagiWorkerCommandRouter::new(handler, responder);
    
    
    
    
    
}