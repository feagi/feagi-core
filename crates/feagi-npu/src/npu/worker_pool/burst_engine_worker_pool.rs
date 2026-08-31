use feagi_data::data_channels::data_channel::DataChannelPair;
use feagi_data::data_channels::data_cycler::DataCycleEndpoint;
use crate::npu::npu_target_frequency::NPUTargetFrequency;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_models::wrapped_indexes::BurstIndex;
use feagi_npu_burst_engines::BurstEngineEnum;
use crate::npu::worker::burst_engine_package::BurstEnginePackage;
use crate::npu::worker::burst_engine_timeout_logic::BurstEngineTimeoutLogic;
use crate::npu::worker::burst_engine_worker::{burst_engine_worker};
use crate::npu::worker_pool::command_and_response::{BurstEngineWorkerFeedback, BurstEngineWorkerPoolCommand};
use crate::npu::worker_pool::pool_struct::{BurstEngineWorkerPool, PoolFeedbackChannel};


/// Entry point for a pool thread: spawns one worker per burst engine, then runs the control loop.
pub fn composable_burst_engine_worker_pool<
    FIQ: FeagiIndexQuantization,
    VisualizationTransmitter: DataCycleEndpoint<u8>,
    MotorTransmitter: DataCycleEndpoint<u8>,
    SensorReceiver: DataCycleEndpoint<u8>,
>(
    burst_engine_packages: Vec<BurstEnginePackage<
        FIQ,
        VisualizationTransmitter,
        MotorTransmitter,
        SensorReceiver
    >>,
    feedback_channels: PoolFeedbackChannel<FIQ>,
    initial_frequency: NPUTargetFrequency,
    starting_burst_index: BurstIndex<FIQ::BurstIndexQuant>,
    timeout_config: BurstEngineTimeoutLogic
) {
    std::thread::scope(|scope| {

        // Init the worker threads

        let mut worker_command_transmitters = Vec::with_capacity(burst_engine_packages.len());
        let mut worker_response_receivers = Vec::with_capacity(burst_engine_packages.len());

        // Each worker owns exactly one burst engine and lives for the whole scope; the scope joins
        // every spawned worker on exit, so we do not need to retain the join handles here.
        for burst_engine_package in burst_engine_packages {
            const COMMAND_BUFFER_SIZE: usize = 1;

            let (worker_command_transmitter, worker_command_receiver) = DataChannelPair::new_pair(COMMAND_BUFFER_SIZE);
            let (worker_response_transmitter, worker_response_receiver) =  DataChannelPair::new_pair(COMMAND_BUFFER_SIZE);


            scope.spawn(move || burst_engine_worker(burst_engine_package, worker_command_receiver, worker_response_transmitter, timeout_config.clone()));
            worker_command_transmitters.push(worker_command_transmitter);
            worker_response_receivers.push(worker_response_receiver);
        }

        let final_return = loop {




        }



    })
}



























/*
/// Entry point for a pool thread: spawns one worker per burst engine, then runs the control loop.
pub fn composable_burst_engine_worker_pool<FIQ: FeagiIndexQuantization>(
    burst_engines: Vec<BurstEngineEnum<FIQ>>, // TODO further layer different quant level as enum here
    feedback_channels: PoolFeedbackChannel<FIQ>,
    initial_frequency: NPUTargetFrequency,
    timeout_config: BurstEngineTimeoutLogic
) {
    std::thread::scope(|scope| {
        let mut worker_commanders = Vec::with_capacity(burst_engines.len());

        // Each worker owns exactly one burst engine and lives for the whole scope; the scope joins
        // every spawned worker on exit, so we do not need to retain the join handles here.
        for burst_engine in burst_engines {
            const COMMAND_BUFFER_SIZE: usize = 1;

            let (commander, follower) = make_commander_follower::<FIQ>(COMMAND_BUFFER_SIZE, COMMAND_BUFFER_SIZE);
            let a = scope.spawn(move || burst_engine_worker(burst_engine, follower, timeout_config.clone()));
            worker_commanders.push(commander);
        }

        let mut pool = BurstEngineWorkerPool::new(feedback_channels, worker_commanders, initial_frequency);
        pool.run();
    })
}

 */

