use crate::connectome_basis::connectome_editing::{ConnectomeDispatch, ConnectomeEditResultantEnum, ConnectomeEditStepEnum};
use crate::connectome_basis::engine::EngineChangeCommandOtherError;
use feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
use feagi_basis::parallelism::feagi_worker::worker_internals::feagi_worker_command_processor::FeagiWorkerCommandProcessor;
use feagi_basis::target_frequency::TargetFrequency;
use feagi_connectome_neural::neural_basis::engine::BurstEngine;
use std::marker::PhantomData;

/// Custom requests to the single engine neuron processor
pub enum NeuronProcessorSingleEngineRequest<FIQ: FeagiIndexQuantization> {
    /// Set the rate the burst loop ticks at. Applies from the next tick
    BurstRate(TargetFrequency),
    /// Set the rate bursts send visualization at. Bursts never run faster than the burst rate, so
    /// a visualization rate above it means every burst sends visualization
    VisualizationRate(TargetFrequency),
    /// Apply these connectome edit steps in order. Stops at the first step that fails. Pauses the
    /// worker, which stays paused afterward, even if a step failed, until sent `Resume`
    ConnectomeEdits(Vec<ConnectomeEditStepEnum<FIQ, ConnectomeDispatch>>),
    /// Stop bursting until resumed. While paused the worker only waits for and handles commands,
    /// so its caller can edit the connectome and update its own state before bursting continues
    Pause,
    /// Continue bursting after a pause
    Resume,
}

/// Custom responses from the single engine neuron processor, one per kind of request
pub enum NeuronProcessorSingleEngineResponse<FIQ: FeagiIndexQuantization> {
    /// The burst rate was set
    BurstRate,
    /// The visualization rate was set
    VisualizationRate,
    /// What each edit step that was attempted returned, in order. If the last one failed, the
    /// steps after it were not attempted
    ConnectomeEdits(Vec<Result<ConnectomeEditResultantEnum<FIQ>, EngineChangeCommandOtherError>>),
    /// Bursting is paused. Also the reply if it already was
    Paused,
    /// Bursting has resumed. Also the reply if it was not paused
    Resumed,
}



#[doc(hidden)]
/// The state of the single engine neuron processor, which its commands act upon
pub struct NeuronProcessorSingleEngineCommandHandler<FIQ: FeagiIndexQuantization, Engine: BurstEngine<FIQ, ConnectomeDispatch, ()>> {
    pub(crate) target_frequency: TargetFrequency,
    pub(crate) visualization_frequency: TargetFrequency,
    /// While paused, no bursts run and the worker only waits for commands
    pub(crate) paused: bool,
    pub(crate) engine: Engine,
    pub(crate) _fiq: PhantomData<fn() -> FIQ>,
}


impl<FIQ: FeagiIndexQuantization, Engine: BurstEngine<FIQ, ConnectomeDispatch, ()>> FeagiWorkerCommandProcessor
for NeuronProcessorSingleEngineCommandHandler<FIQ, Engine>
{
    type CustomRequest = NeuronProcessorSingleEngineRequest<FIQ>;
    type CustomResponse = NeuronProcessorSingleEngineResponse<FIQ>;

    /// The burst loop reads both rates each tick, so a rate set here applies from the next tick
    async fn on_custom(&mut self, request: Self::CustomRequest) -> Self::CustomResponse {
        match request {
            NeuronProcessorSingleEngineRequest::BurstRate(burst_rate) => {
                self.target_frequency = burst_rate;
                NeuronProcessorSingleEngineResponse::BurstRate
            }
            NeuronProcessorSingleEngineRequest::VisualizationRate(visualization_rate) => {
                self.visualization_frequency = visualization_rate;
                NeuronProcessorSingleEngineResponse::VisualizationRate
            }
            NeuronProcessorSingleEngineRequest::ConnectomeEdits(steps) => {
                self.paused = true; // NOTE: System will pause when edits are submitted, and must be unpaused after
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

impl<FIQ: FeagiIndexQuantization, Engine: BurstEngine<FIQ, ConnectomeDispatch, ()>> NeuronProcessorSingleEngineCommandHandler<FIQ, Engine> {
    /// Relays each edit step to the engine in order, stopping at the first that fails
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

