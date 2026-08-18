use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_models::wrapped_indexes::BurstIndex;
use crate::standard::npu::burst_engine::burst_engine_communication::{EngineConnectomeEditRequest, EngineConnectomeEditResponse, KernelCommand};
// TODO maybe edit requests should return some sort of one shot channel?

/// Has a burst engine do something.
///
/// `RunKernel` carries the burst index the pool is currently orchestrating so that responses
/// can be correlated back to a specific burst.
pub enum BurstEngineWorkerCommand<FIQ: FeagiIndexQuantization> {
    RunKernel {
        burst_index: BurstIndex<FIQ::GlobalBurstIndexQuant>,
        kernel: KernelCommand,
    },
    EngineConnectomeEdits(Vec<EngineConnectomeEditRequest<FIQ>>),
    Stop,
}


/// After a burst engine does a thing, this is the response.
///
/// `KernelRan` echoes back the burst index it corresponds to so the pool can verify ordering.
pub enum BurstEngineWorkerResponse<FIQ: FeagiIndexQuantization> {
    KernelRan {
        burst_index: BurstIndex<FIQ::GlobalBurstIndexQuant>,
        // TODO carry Result<(), FeagiBurstEngineError> or metrics
    },
    EngineConnectomeEdited(Vec<EngineConnectomeEditResponse<FIQ>>),
    Stopped,
}
