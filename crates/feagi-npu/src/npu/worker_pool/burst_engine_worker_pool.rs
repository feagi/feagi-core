use crate::npu::npu_target_frequency::NPUTargetFrequency;
use feagi_data::channels::channels_flume::OuterFlumeChannelPair;
use feagi_models::quantization_levels::burst_engine_index_quantization::BurstEngineIndexQuantizationQuantizationNormal;
use feagi_models::quantization_levels::neuron_processing_unit_index_quantization::NeuronProcessingUnitIndexQuantization;
use feagi_npu_burst_engines::BurstEngineEnum;
use crate::npu::worker::command_and_response::{BurstEngineWorkerCommand, BurstEngineWorkerResponse};
use crate::npu::worker::burst_engine_worker::{burst_engine_worker, make_commander_follower};
use crate::npu::worker_pool::command_and_response::{BurstEngineWorkerFeedback, BurstEngineWorkerPoolCommand};
use crate::npu::worker_pool::pool_struct::{BurstEngineWorkerPool, PoolFeedbackChannel};

pub type BurstEngineWorkerCommander<NPUIQ: NeuronProcessingUnitIndexQuantization> =
    OuterFlumeChannelPair<BurstEngineWorkerPoolCommand<NPUIQ>, BurstEngineWorkerFeedback<NPUIQ>>;

/// Entry point for a pool thread: spawns one worker per burst engine, then runs the control loop.
pub fn composable_burst_engine_worker_pool<NPUIQ: NeuronProcessingUnitIndexQuantization>(
    burst_engines: Vec<BurstEngineEnum<NPUIQ, BurstEngineIndexQuantizationQuantizationNormal>>,
    feedback_channels: PoolFeedbackChannel<NPUIQ>,
    initial_frequency: NPUTargetFrequency,
) {
    std::thread::scope(|scope| {
        let mut worker_commanders = Vec::with_capacity(burst_engines.len());

        // Each worker owns exactly one burst engine and lives for the whole scope; the scope joins
        // every spawned worker on exit, so we do not need to retain the join handles here.
        for burst_engine in burst_engines {
            const COMMAND_BUFFER_SIZE: usize = 1;

            let (commander, follower) = make_commander_follower::<NPUIQ>(COMMAND_BUFFER_SIZE, COMMAND_BUFFER_SIZE);
            scope.spawn(move || burst_engine_worker(burst_engine, follower));
            worker_commanders.push(commander);
        }

        let mut pool = BurstEngineWorkerPool::new(feedback_channels, worker_commanders, initial_frequency);
        pool.run();
    })
}

