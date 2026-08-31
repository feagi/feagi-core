use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_models::wrapped_indexes::BurstIndex;
use feagi_npu_burst_engines::BurstEnginePackage;
use feagi_npu_burst_engines::feagi_npu_burst_core::burst_engine_definitions::burst_phase_output::ItemRequestingAllocationIncrease;
use feagi_npu_burst_engines::feagi_npu_burst_core::burst_engine_definitions::burst_phases::RunBurstPhase;
use feagi_npu_burst_engines::feagi_npu_burst_core::errors::BurstEngineWorkerError;


/// This command awakes a worker to have it execute the burst in some way
pub enum BurstEngineWorkerCommand<FIQ: FeagiIndexQuantization> {
    /// Run the burst engine, either the default full burst or a specific phase.
    RunPhases {
        burst_index: BurstIndex<FIQ::BurstIndexQuant>,
        phase: RunBurstPhase,
    },

    /// Burst index is overflowing, its being changed now to this
    BurstIndexRollback {
        new_burst_index: BurstIndex<FIQ::BurstIndexQuant>,
        previous_burst_index: BurstIndex<FIQ::BurstIndexQuant>,
    },

    /// Stop (safely) this worker https://www.youtube.com/watch?v=x012BnKWi3g&t=97s
    CommitSudoku,

    // TODO insert a new agent

    // TODO Below must be Composable
    /// Reorder structures in memory to order unused memory at the end, then free it
    DefragmentAndReleaseUnusedIndexes,



    /// Submit connectome change requests


    // TODO unique dynamic commands (defrag)
    // TODO Unique Composable Commands
}



/// The responses that come from the Burst Engine Worker at the end of each burst
pub enum BurstEngineWorkerResponse<FIQ: FeagiIndexQuantization> {
    NoFurtherActionNeeded,
    /// Hes dead, jim
    BrainDeathTriggered,
    BurstEngineNeedsAttention(Vec<ItemRequestingAllocationIncrease<FIQ>>), // TODO dynamic
}




/// When a burst engine worker closes for whatever reason, this object is returned
pub struct BurstEngineWorkerConclusion<FIQ: FeagiIndexQuantization, EnginePackage: BurstEnginePackage<FIQ>> {
    pub burst_engine_package: EnginePackage,
    pub error: Option<BurstEngineWorkerError>,
    _p: core::marker::PhantomData<FIQ>,
}

impl<FIQ: FeagiIndexQuantization, EnginePackage: BurstEnginePackage<FIQ>> BurstEngineWorkerConclusion<FIQ, EnginePackage> {
    pub fn new(burst_engine_package: EnginePackage, error: Option<BurstEngineWorkerError>) -> Self {
        Self {
            burst_engine_package,
            error,
            _p: core::marker::PhantomData,
        }
    }
}
