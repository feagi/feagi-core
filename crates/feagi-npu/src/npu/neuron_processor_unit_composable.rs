use core::time::Duration;
use feagi_data::bidirectional_channel_queue::BidirectionalChannelSide;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::npu::burst_engine::composable_implementations::tokio_rayon::tokio_rayon_burst_engine::TokioRayonBurstEngine;
use crate::npu::burst_engine::ComposableBurstEngineEnum;
use crate::npu::burst_engine_worker::burst_engine_commands::BurstEngineWorkerCommand;
use crate::npu::NPUTargetFrequency::NPUTargetFrequency;

pub struct NeuronProcessingUnitComposable<FIQ: FeagiIndexQuantization> {
    npu_state: NPUState,
    worker_pool: NPUWorkerPool<FIQ>,

}

impl<FIQ: FeagiIndexQuantization> NeuronProcessingUnitComposable<FIQ> {

    /// Creates a new NPU with burst engines, but does not start anything
    pub fn new() -> Self {

        // TODO take in burst engines as a parameter, for now defined for you

        let burst_engine = ComposableBurstEngineEnum::TokioRayonBurstEngine(
            TokioRayonBurstEngine::new()
        );

        // TODO multiple engines

        let worker_pool = NPUWorkerPool::Frozen(burst_engine);

        Self {
            npu_state: NPUState::Paused,
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
            NPUWorkerPool::Running { handle, pool_command_queue, target_frequency } => {
                // TODO send stop command
            }
        }
    }
    
    pub fn start_engines(&mut self, set_target_frequency: NPUTargetFrequency) {
        
        match self.worker_pool {
            NPUWorkerPool::None => {
                // TODO ???
            }
            NPUWorkerPool::Frozen(engines) => {
                
            }
            NPUWorkerPool::Running { handle, pool_command_queue, target_frequency } => {
                // only update if we have a new frequency
                if set_target_frequency == target_frequency {
                    return;
                }
                
                // TODO
                
            }
        }
        
    }
}

pub enum NPUState {
    Failed,
    Paused,
    Running{ target_frequency: NPUTargetFrequency },
}


enum NPUWorkerPool<'a, FIQ: FeagiIndexQuantization> {
    None,
    Frozen(ComposableBurstEngineEnum<FIQ>),
    Running{ // ?
        handle: (),
        pool_command_queue: BidirectionalChannelSide<'a, BurstEngineWorkerCommand>,
        target_frequency: NPUTargetFrequency,
    },
}









