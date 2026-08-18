use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::standard::npu::burst_engine_worker::burst_engine_worker_communication::{
    BurstEngineWorkerCommand, BurstEngineWorkerResponse,
};

/// Channels owned by a burst engine worker (its side of the wiring to the pool).
pub struct BurstEngineWorkerChannels<FIQ: FeagiIndexQuantization> {
    pub command_rx: flume::Receiver<BurstEngineWorkerCommand<FIQ>>,
    pub response_tx: flume::Sender<BurstEngineWorkerResponse<FIQ>>,
    // TODO data injection channels (sensor / force-fire) and extraction channels
    //      (motor / visualization) live here once their message types are settled.
    // TODO peer channels to other workers for mid-tick data exchange.
}

/// Pool-side of the wiring to a single worker.
pub struct BurstEngineWorkerPoolSideChannels<FIQ: FeagiIndexQuantization> {
    pub command_tx: flume::Sender<BurstEngineWorkerCommand<FIQ>>,
    pub response_rx: flume::Receiver<BurstEngineWorkerResponse<FIQ>>,
}

impl<FIQ: FeagiIndexQuantization> BurstEngineWorkerChannels<FIQ> {
    /// Create paired channels for a new worker, returning `(pool_side, worker_side)`.
    pub fn new_pair(
        capacity: usize,
    ) -> (BurstEngineWorkerPoolSideChannels<FIQ>, Self) {
        let (command_tx, command_rx) = flume::bounded(capacity);
        let (response_tx, response_rx) = flume::bounded(capacity);
        (
            BurstEngineWorkerPoolSideChannels {
                command_tx,
                response_rx,
            },
            Self {
                command_rx,
                response_tx,
            },
        )
    }
}
