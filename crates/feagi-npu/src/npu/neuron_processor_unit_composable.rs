use crate::npu::burst_engine::composable_implementations::tokio_rayon::tokio_rayon_burst_engine::TokioRayonBurstEngine;
use crate::npu::burst_engine::ComposableBurstEngineEnum;
use crate::npu::burst_engine_worker::burst_engine_worker_pool::{
    burst_engine_worker_pool, default_burst_coordinator_loop, BurstEngineWorkerChannels,
};
use crate::npu::npu_target_frequency::NPUTargetFrequency;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

pub struct NeuronProcessingUnitComposable<FIQ: FeagiIndexQuantization> {
    npu_state: NPUState,
    worker_channels: Vec<BurstEngineWorkerChannels>,
    frozen_engine_pool: NPUWorkerPool<FIQ>,
}

impl<FIQ: FeagiIndexQuantization + Send + 'static> NeuronProcessingUnitComposable<FIQ> {
    /// Creates a new NPU with burst engines, but does not start anything
    pub fn new() -> Self {
        // TODO take in burst engines as a parameter, for now defined for you
        let burst_engine = ComposableBurstEngineEnum::TokioRayonBurstEngine(TokioRayonBurstEngine::new());

        let frozen_engine_pool = NPUWorkerPool::Frozen(vec![burst_engine]);

        Self {
            npu_state: NPUState::Paused,
            worker_channels: vec![BurstEngineWorkerChannels::new()],
            frozen_engine_pool,
        }
    }

    pub fn stop_engines(&mut self) {
        match &mut self.frozen_engine_pool {
            NPUWorkerPool::None => {}
            NPUWorkerPool::Frozen(_) => {}
            NPUWorkerPool::Running { .. } => {
                // TODO stop all workers
            }
        }
    }

    pub fn start_engines(&mut self, set_target_frequency: NPUTargetFrequency) {

        return ();

        // TODO address

        match &mut self.frozen_engine_pool {
            NPUWorkerPool::None => {
                // TODO ???
            }
            NPUWorkerPool::Frozen(engines) => {
                if self.worker_channels.len() != engines.len() {
                    self.worker_channels.resize_with(engines.len(), BurstEngineWorkerChannels::new);
                }

                let burst_engines = core::mem::take(engines);
                self.npu_state = NPUState::Running {
                    target_frequency: set_target_frequency,
                };
                self.frozen_engine_pool = NPUWorkerPool::Running {
                    target_frequency: set_target_frequency,
                };

                burst_engine_worker_pool(&mut self.worker_channels, burst_engines, |coordinator_sides| {
                    default_burst_coordinator_loop(coordinator_sides);
                });
            }
            NPUWorkerPool::Running { target_frequency } => {
                if set_target_frequency == *target_frequency {
                    return;
                }

                // TODO update frequency while running
            }
        }
    }
}

pub enum NPUState {
    Failed,
    Paused,
    Running { target_frequency: NPUTargetFrequency },
}

enum NPUWorkerPool<FIQ: FeagiIndexQuantization> {
    None,
    Frozen(Vec<ComposableBurstEngineEnum<FIQ>>),
    Running { target_frequency: NPUTargetFrequency },
}
