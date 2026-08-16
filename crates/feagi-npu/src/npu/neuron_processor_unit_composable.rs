use feagi_data::bidirectional_channel_queue::{BidirectionalChannelSide, MpscBidirectionalChannelSide};
use crate::npu::burst_engine::composable_implementations::tokio_rayon::tokio_rayon_burst_engine::TokioRayonBurstEngine;
use crate::npu::burst_engine::ComposableBurstEngineEnum;
use crate::npu::burst_engine_worker::burst_engine_worker_pool::{
    burst_engine_worker_pool, BurstEngineWorkerChannels, BurstEngineWorkerCoordinatorSide,
};
use crate::npu::npu_target_frequency::NPUTargetFrequency;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::wnpu::agents::data_exchange::force_fire::VoxelForceFire;
use crate::wnpu::agents::data_exchange::visualization::VoxelVisualization;
use crate::wnpu::agents::data_exchange::voxel_potentials::CorticalAreaVoxelPotentials;

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

    pub fn start_engines(
        &mut self,
        set_target_frequency: NPUTargetFrequency,
        inputs_sensors: &mut [MpscBidirectionalChannelSide<CorticalAreaVoxelPotentials<u32, f32>>],
        inputs_force_fire: &mut [MpscBidirectionalChannelSide<VoxelForceFire<u32>>],
        outputs_motors: &mut [MpscBidirectionalChannelSide<CorticalAreaVoxelPotentials<u32, f32>>],
        outputs_visualization: &mut [MpscBidirectionalChannelSide<VoxelVisualization<u32>>],
    ) {

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
                    Self::proxy_burst_coordinator_loop(
                        coordinator_sides,
                        inputs_sensors,
                        inputs_force_fire,
                        outputs_motors,
                        outputs_visualization,
                    );
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

    fn proxy_burst_coordinator_loop(
        coordinator_sides: &mut [BurstEngineWorkerCoordinatorSide<'_>],
        inputs_sensors: &mut [MpscBidirectionalChannelSide<CorticalAreaVoxelPotentials<u32, f32>>],
        inputs_force_fire: &mut [MpscBidirectionalChannelSide<VoxelForceFire<u32>>],
        outputs_motors: &mut [MpscBidirectionalChannelSide<CorticalAreaVoxelPotentials<u32, f32>>],
        outputs_visualization: &mut [MpscBidirectionalChannelSide<VoxelVisualization<u32>>],
    ) {
        loop {
            // Current implementation intentionally assumes a single worker.
            let Some(coordinator_side) = coordinator_sides.first_mut() else {
                return;
            };

            for sensor_input in inputs_sensors.iter_mut() {
                while let Some(sensor_data) = sensor_input.dequeue() {
                    let _ = coordinator_side.incoming_sensor.enqueue(sensor_data);
                }
            }

            for force_fire_input in inputs_force_fire.iter_mut() {
                while let Some(force_fire_data) = force_fire_input.dequeue() {
                    let _ = coordinator_side.incoming_force_fire.enqueue(force_fire_data);
                }
            }

            while let Some(motor_data) = coordinator_side.outgoing_motor.dequeue() {
                let mut pending = Some(motor_data);
                for motor_output in outputs_motors.iter_mut() {
                    let Some(data) = pending.take() else {
                        break;
                    };
                    match motor_output.enqueue(data) {
                        Ok(()) => break,
                        Err(std::sync::mpsc::TrySendError::Full(data))
                        | Err(std::sync::mpsc::TrySendError::Disconnected(data)) => {
                            pending = Some(data);
                        }
                    }
                }
            }

            while let Some(visualization_data) = coordinator_side.outgoing_visualization.dequeue() {
                let mut pending = Some(visualization_data);
                for visualization_output in outputs_visualization.iter_mut() {
                    let Some(data) = pending.take() else {
                        break;
                    };
                    match visualization_output.enqueue(data) {
                        Ok(()) => break,
                        Err(std::sync::mpsc::TrySendError::Full(data))
                        | Err(std::sync::mpsc::TrySendError::Disconnected(data)) => {
                            pending = Some(data);
                        }
                    }
                }
            }

            std::thread::yield_now();
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




pub struct AgentRegistrationChannels<'a> {
    pub visualization: Option<BidirectionalChannelSide<'a, ()>>,
    // TODO other types of data exchange
}