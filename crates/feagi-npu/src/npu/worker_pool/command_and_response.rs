use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_npu_burst_engines::BurstEngineEnum;
use feagi_npu_burst_engines::feagi_npu_burst_core::burst_engine_definitions::burst_phase_output::ItemRequestingAllocationIncrease;
use feagi_npu_burst_engines::feagi_npu_burst_core::errors::BurstEngineWorkerPoolError;
use crate::npu::npu_target_frequency::NPUTargetFrequency;
use crate::npu::worker::command_and_response::BurstEngineWorkerCommand;

/// The commands to control what the burst engine workers in the pool should do
pub enum BurstEngineWorkerPoolCommand<FIQ: FeagiIndexQuantization> {
    /// Sends a specific command to a specific burst engine, ignoring frequency rate limiting
    SpecificEngineCommand {
        burst_engine_index: u16,
        command: BurstEngineWorkerCommand<FIQ>,
    },
    Pause,
    SetBurstFrequency(NPUTargetFrequency),
    // /// Run a single full burst, without any frequency rate limiting
    // RunSingleFullBurst,
    // BurstIndexRollback {
    //    burst_index: BurstIndex<FIQ::BurstIndexQuant>,
    //},
    StopAllWorkersAndClose,
}

/// The pool does not return data every burst, only to alert about something
pub enum BurstEngineWorkerFeedback<FIQ: FeagiIndexQuantization> {
    BrainDeathTriggered,
    // TODO workers exceeded time
    BurstIndexAboutToOverflow(FIQ),
    //MoreAllocationNeeded(Vec<Option<Vec<ItemRequestingAllocationIncrease<FIQ>>>>), // TODO composable
}

/// Returned when a pool closes for any reason
pub struct BurstEnginePoolConclusion<FIQ: FeagiIndexQuantization> {
    pub burst_engines: Vec<BurstEngineEnum<FIQ>>,
    pub error: Option<BurstEngineWorkerPoolError>,
}