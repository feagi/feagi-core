use feagi_models::quantization_levels::neuron_processing_unit_index_quantization::NeuronProcessingUnitIndexQuantization;
use crate::npu::npu_target_frequency::NPUTargetFrequency;
use crate::npu::worker::command_and_response::BurstEngineWorkerCommand;

/// The commands to control what the burst engine workers in the pool should do
pub enum BurstEngineWorkerPoolCommand<NPUIQ: NeuronProcessingUnitIndexQuantization> {
    /// Sends a specific command to a specific burst engine, ignoring frequency rate limiting
    SpecificEngineCommand {
        burst_engine_index: u16,
        command: BurstEngineWorkerCommand<NPUIQ>,
    },
    Pause,
    SetBurstFrequency(NPUTargetFrequency),
    // /// Run a single full burst, without any frequency rate limiting
    // RunSingleFullBurst,
    // BurstIndexRollback {
    //    burst_index: BurstIndex<NPUIQ::BurstIndexQuant>,
    //},
    StopAllWorkersAndClose,
}

/// The pool does not return data every burst, only to alert about something
pub enum BurstEngineWorkerFeedback<NPUIQ: NeuronProcessingUnitIndexQuantization> {
    SafelyStopped,
    // TODO workers exceeded time
    BrainDeathTriggered,
    BurstIndexAboutToOverflow(NPUIQ),
    /// Hes dead, jim
    WorkerCrashed(u16, &'static str),
    UnknownFailure(&'static str),
}