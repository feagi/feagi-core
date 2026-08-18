use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::standard::npu::burst_engine_worker::burst_engine_worker_channels::BurstEngineWorkerPoolSideChannels;
use crate::standard::npu::npu_communication::NPUCommand;

/// Channels owned by the pool loop.
pub struct BurstEngineWorkerPoolChannels<FIQ: FeagiIndexQuantization> {
    pub npu_command_rx: flume::Receiver<NPUCommand>,
    pub worker_channels: Vec<BurstEngineWorkerPoolSideChannels<FIQ>>,
}

impl<FIQ: FeagiIndexQuantization> BurstEngineWorkerPoolChannels<FIQ> {
    /// Bundle a set of pre-created per-worker channel handles into a pool channel set,
    /// and create the wrappe  pool command channel pair.
    pub fn new(
        npu_command_capacity: usize,
        worker_channels: Vec<BurstEngineWorkerPoolSideChannels<FIQ>>,
    ) -> (Self, flume::Sender<NPUCommand>) {
        let (npu_command_tx, npu_command_rx) = flume::bounded(npu_command_capacity);
        (
            Self {
                npu_command_rx,
                worker_channels,
            },
            npu_command_tx,
        )
    }
}
