use feagi_models::quantization_levels::neuron_processing_unit_index_quantization::NeuronProcessingUnitIndexQuantization;
use feagi_models::wrapped_indexes::BurstIndex;
use feagi_npu_burst_engines::feagi_npu_burst_core::burst_engine_definitions::burst_phases::RunBurstPhase;

// TODO feature gated composable calls

/// This command awakes a worker to have it execute the burst in some way
pub enum BurstEngineWorkerCommand<NPUIQ: NeuronProcessingUnitIndexQuantization> {
    /// Run the burst engine, either the default full burst or a specific phase.
    RunPhases {
        burst_index: BurstIndex<NPUIQ::BurstIndexQuant>,
        phase: RunBurstPhase,
    },
    /// Burst index is overflowing, its being changed now to this
    BurstIndexRollback { burst_index: BurstIndex<NPUIQ::BurstIndexQuant> },
    /// Hard terminate this worker https://www.youtube.com/watch?v=x012BnKWi3g&t=97s
    CommitSudoku,
    // TODO Unique Composable Commands
}


/// The responses that come from the Burst Engine Worker at the end of each burst
pub enum BurstEngineWorkerResponse<NPUIQ: NeuronProcessingUnitIndexQuantization> {
    KernelRan {
        burst_index: BurstIndex<NPUIQ::BurstIndexQuant>,
        // TODO carry Result<(), FeagiBurstEngineError> or metrics
    },
    Stopped,
    /// Hes dead, jim
    BrainDeathTriggered,
    /// While probably not the cause, lets blame Windows for this anyways
    Crashed(&'static str),
    //EngineConnectomeEdited(Vec<EngineConnectomeEditResponse<FIQ>>),
    // TODO Unique Composable Responses
}
