use crate::npu::composable_std::composable_burst_engine_worker::{
    composable_burst_engine_worker, make_commander_follower, ComposableBurstEngineWorkerCommand,
};
use crate::npu_3::npu_target_frequency::NPUTargetFrequency;
use feagi_data::channels::channels::{ChannelPair, ChannelReceivingError};
use feagi_data::channels::channels_flume::{InnerFlumeChannelPair, OuterFlumeChannelPair};
use feagi_models::quantization_levels::burst_engine_index_quantization::BurstEngineIndexQuantizationQuantizationNormal;
use feagi_models::quantization_levels::neuron_processing_unit_index_quantization::NeuronProcessingUnitIndexQuantization;
use feagi_models::wrapped_indexes::BurstIndex;
use feagi_npu_burst_engines::ComposableBurstEngineEnum;

pub type BurstEngineWorkerCommander<NPUIQ: NeuronProcessingUnitIndexQuantization> =
    OuterFlumeChannelPair<ComposableBurstEngineWorkerPoolCommand<NPUIQ>, ComposableBurstEngineWorkerFeedback<NPUIQ>>;

pub fn composable_burst_engine_worker_pool<NPUIQ: NeuronProcessingUnitIndexQuantization>(
    mut burst_engines: Vec<ComposableBurstEngineEnum<NPUIQ, BurstEngineIndexQuantizationQuantizationNormal>>,
    mut feedback_channels: InnerFlumeChannelPair<ComposableBurstEngineWorkerFeedback<NPUIQ>, ComposableBurstEngineWorkerPoolCommand<NPUIQ>>,
    initial_frequency: NPUTargetFrequency,
) {
    std::thread::scope(|scope| {
        let mut workers = Vec::with_capacity(burst_engines.len());
        let mut worker_commanders = Vec::with_capacity(burst_engines.len());
        let mut frequency = initial_frequency;
        let mut bursts_paused: bool = false;
        let mut burst_index: BurstIndex<NPUIQ::BurstIndexQuant> = BurstIndex::QUANT_MAX / (BurstIndex::QUANT_ONE + BurstIndex::QUANT_ONE);

        for burst_engine in &mut burst_engines {
            const COMMAND_BUFFER_SIZE: usize = 1;

            let (outer, inner) = make_commander_follower::<NPUIQ>(COMMAND_BUFFER_SIZE, COMMAND_BUFFER_SIZE);

            let worker_handle = scope.spawn(move || composable_burst_engine_worker(*burst_engine, inner));
            workers.push(worker_handle);
            worker_commanders.push(outer);
        }



        loop {
            let possible_command = if bursts_paused {
                let command = feedback_channels.block_receive();
                if let Err(e) = command {
                    match e {
                        ChannelReceivingError::ReceiveFailed(_) => {
                            // Something went wrong
                            _ = feedback_channels.try_send(ComposableBurstEngineWorkerFeedback::UnknownFailure("Pool receiver failure"));
                            break;
                        }
                        ChannelReceivingError::ReceiveTimeout(_) => {
                            // Not sure how we would get this on block receive, skip and restart
                            continue;
                        }
                    }
                }
                Some(command.unwrap())
                // If its not an error, assume its a command that could unpause us
            } else {
                // We are unpaused
                let command = feedback_channels.try_receive();
                if let Err(e) = command {
                    match e {
                        ChannelReceivingError::ReceiveFailed(_) => {
                            // Something went wrong
                            _ = feedback_channels.try_send(ComposableBurstEngineWorkerFeedback::UnknownFailure("Pool receiver failure"));
                            break;
                        }
                        ChannelReceivingError::ReceiveTimeout(_) => {
                            // Not sure how we would get this on block receive, skip and restart
                            continue;
                        }
                    }
                }
                command.unwrap()
            };

            if let Some(command) = possible_command {
                // We got a command to change something

                match command {
                    ComposableBurstEngineWorkerPoolCommand::SpecificEngineCommand {
                        burst_engine_index,
                        command } => {

                        let worker_commander = worker_commanders.get_mut(burst_engine_index as usize);

                        if let Some(commander) = worker_commander {
                            commander.block_send(command); // TODO error handling
                            let response = commander.block_receive().unwrap();
                            continue;
                        }
                        // Why did NPU send us an invalid index?
                        _ = feedback_channels.try_send(
                            ComposableBurstEngineWorkerFeedback::UnknownFailure("Invalid Burst Engine Index Sent")
                        );
                        break;
                    }
                    ComposableBurstEngineWorkerPoolCommand::Pause => {
                        bursts_paused = true;
                        continue;
                    }
                    ComposableBurstEngineWorkerPoolCommand::SetBurstFrequency(f) => {
                        bursts_paused = false;
                        frequency = f;
                        // Just continue the loop
                    }
                    ComposableBurstEngineWorkerPoolCommand::StopAllWorkersAndClose => {
                        for worker_commander in &mut worker_commanders {
                            worker_commander.block_send(
                                ComposableBurstEngineWorkerCommand::CommitSudoku
                            ); //TODO error handling!

                        }
                        break;
                    }
                }
                continue;
            }
            // No command, continue normally

            // TODO increment burst index

            let current_time = std::time::Instant::now();

            // Send command to all
            for worker_commander in &mut worker_commanders {
                _ =worker_commander.try_send(ComposableBurstEngineWorkerCommand::RunPhases {
                    burst_index: burst_index,
                    phase: Default::default(),
                });
            }

            // block until all are done
            for worker_commander in &mut worker_commanders {
                _ = worker_commander.block_receive()
            }


        }
    })
}

/// The commands to control what the burst engine workers in the pool should do
pub enum ComposableBurstEngineWorkerPoolCommand<NPUIQ: NeuronProcessingUnitIndexQuantization> {
    /// Sends a specific command to a specific burst engine, ignoring frequency rate limiting
    SpecificEngineCommand {
        burst_engine_index: u16,
        command: ComposableBurstEngineWorkerCommand<NPUIQ>,
    },
    Pause,
    SetBurstFrequency(NPUTargetFrequency),
    // /// Run a single full burst, without any frequency rate limiting
    // RunSingleFullBurst,
    // BurstIndexRollback {
    //    burst_index: BurstIndex<NPUIQ::BurstIndexQuant>,
    //},
    StopAllWorkersAndClose,
}

/// The pool does not return data every burst, only to alert about something
pub enum ComposableBurstEngineWorkerFeedback<NPUIQ: NeuronProcessingUnitIndexQuantization> {
    SafelyStopped,
    BrainDeathTriggered,
    /// Hes dead, jim
    WorkerCrashed(u16, &'static str),
    UnknownFailure(&'static str),
}
