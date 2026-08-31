use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_models::wrapped_indexes::BurstIndex;
use feagi_npu_burst_engines::BurstEngineEnum;
use feagi_npu_burst_engines::feagi_npu_burst_core::errors::BurstEngineWorkerPoolError;
use crate::npu::npu_target_frequency::NPUTargetFrequency;
use crate::npu::worker::communication::BurstEngineWorkerCommand;

/// The commands to control what the burst engine workers in the pool should do
pub enum BurstEngineWorkerPoolCommand<FIQ: FeagiIndexQuantization> {
    /// Sends a specific command to a specific burst engine, ignoring frequency rate limiting
    SpecificEngineCommand {
        burst_engine_index: u16,
        command: BurstEngineWorkerCommand<FIQ>,
    },
    /// Pause the engines but keep them loaded
    Pause,
    /// (Resume and) set the rate of bursts
    SetBurstFrequency(NPUTargetFrequency),
    /// Run bursts without any rate limiting
    RunBurstsFullSpeed,
    /// Burst index is overflowing, its being changed now to this
    BurstIndexRollback {
        new_burst_index: BurstIndex<FIQ::BurstIndexQuant>,
        previous_burst_index: BurstIndex<FIQ::BurstIndexQuant>,
    },
    /// Command all engines to killbind
    StopAllWorkersAndClose,
}

/// The pool does not return data every burst, only to proxy alerts from burst engines
pub enum BurstEngineWorkerPoolFeedback<FIQ: FeagiIndexQuantization> {
    /// Takes top priority, commence brain death. Schadenfreude
    BrainDeathTriggered,
    /// Commence safe rollover of burst index
    BurstIndexAboutToOverflow(FIQ),
    /// Composable only, structure in a burst engine is requesting more allocation
    BurstEngineRequiresAction( Vec<(u16, )>)
    
    
    //MoreAllocationNeeded(Vec<Option<Vec<ItemRequestingAllocationIncrease<FIQ>>>>), // TODO composable
}

/// Returned when a pool closes for any reason
pub struct BurstEnginePoolConclusion<FIQ: FeagiIndexQuantization> {
    pub burst_engines: Vec<BurstEngineEnum<FIQ>>,
    pub error: Option<BurstEngineWorkerPoolError>,
}