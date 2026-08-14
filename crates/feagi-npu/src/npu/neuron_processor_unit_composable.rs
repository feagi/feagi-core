use core::time::Duration;
use feagi_data::bidirectional_channel_queue::BidirectionalChannelSide;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::burst_engine::composable_implementations::tokio_rayon::tokio_rayon_burst_engine::TokioRayonBurstEngine;
use crate::burst_engine_enum::ComposableBurstEngineEnum;
use crate::npu::burst_engine_worker::burst_engine_commands::BurstEngineWorkerCommand;
use crate::npu::neuron_processing_unit_commands::BurstFrequency;

pub struct NeuronProcessingUnitComposable<FIQ: FeagiIndexQuantization> {
    target_burst_duration: Duration,
    worker_pool: NPUWorkerPool<FIQ>,

}

impl<FIQ: FeagiIndexQuantization> NeuronProcessingUnitComposable<FIQ> {

    /// Creates a new NPU with burst engines, but does not start anything
    pub fn new(initial_frequency: BurstFrequency) -> Self {

        // TODO take in burst engines as a parameter, for now defined for you

        let burst_engine = ComposableBurstEngineEnum::TokioRayonBurstEngine(
            TokioRayonBurstEngine::new()
        );

        // TODO multiple engines

        let worker_pool = NPUWorkerPool::Frozen(burst_engine);

        let target_burst_duration = core::time::Duration::from_secs_f64(1.0 / initial_frequency);

        Self {
            target_burst_duration,
            worker_pool
        }
    }

    pub fn stop_engines(&mut self) {

        match self.worker_pool {
            NPUWorkerPool::None => {
                // nothing to do
            }
            NPUWorkerPool::Frozen(engines) => {
                // Nothing to do
            }
            NPUWorkerPool::Running { handle, pool_command_queue } => {
                // TODO send stop command
            }
        }



    }
}



enum NPUWorkerPool<'a, FIQ: FeagiIndexQuantization> {
    None,
    Frozen(ComposableBurstEngineEnum<FIQ>),
    Running{ // ?
        handle: (),
        pool_command_queue: BidirectionalChannelSide<'a, BurstEngineWorkerCommand>,
    },
}









