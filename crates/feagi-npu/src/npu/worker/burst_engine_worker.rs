use feagi_data::data_channels::data_channel::{DataReceiver, DataTransmitter};
use feagi_data::data_channels::data_cycler::DataCycleEndpoint;
use feagi_data::data_channels::errors::ChannelReceivingError;
use crate::npu::worker::burst_engine_timeout_logic::BurstEngineTimeoutLogic;
use crate::npu::worker::command_and_response::{BurstEngineWorkerCommand, BurstEngineWorkerConclusion, BurstEngineWorkerResponse};
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_npu_burst_engines::feagi_npu_burst_core::burst_engine_definitions::burst_engine::BurstEngine;
use feagi_npu_burst_engines::feagi_npu_burst_core::burst_engine_definitions::burst_phase_output::BurstPhaseOutput;
use crate::npu::worker::burst_engine_package::BurstEnginePackage;

/// Execute burst engine, burst by burst, command by command. Return engine when done
pub fn burst_engine_worker<
    FIQ: FeagiIndexQuantization,
    CommandReceiver: DataReceiver<BurstEngineWorkerCommand<FIQ>>,
    ResponseTransmitter: DataTransmitter<BurstEngineWorkerResponse<FIQ>>,
    VisualizationTransmitter: DataCycleEndpoint<u8>,
    MotorTransmitter: DataCycleEndpoint<u8>,
    SensorReceiver: DataCycleEndpoint<u8>,
>
(
    mut burst_engine: BurstEnginePackage<
        FIQ,
        VisualizationTransmitter,
        MotorTransmitter,
        SensorReceiver
    >,
    mut command_receiver: CommandReceiver,
    mut response_transmitter: ResponseTransmitter,
    timeout_logic: BurstEngineTimeoutLogic,
) -> BurstEngineWorkerConclusion<FIQ>
{
    // Run a loop, start by blocking this thread until we get a command. The command will have
    // us execute a burst, make an edit (if engine is composable), or something in between. Try
    // to do the task, and send an output message of completion. Return the burst engine when
    // closing, including during a crash

    let conclusion = loop {
        // Block thread and wait for incoming command
        let incoming_command = match command_receiver.block_receive() {
            Ok(command) => command,
            Err(ChannelReceivingError::ReceiveFailed(e)) => {
                /// Shouldn't be possible? Just wait for the thread
                continue;
            }
            Err(ChannelReceivingError::ReceiveTimeout(e)) => {
                // We shouldn't be using timeouts here? This should be impossible
                panic!("Burst Engine Channel Failed to Receive before Timeout!");
            }
        };

        // We have a command. Execute it. Send response if needed, or shut down if theres an issue
        let option_error = match incoming_command {
            BurstEngineWorkerCommand::RunPhases { burst_index, phase } => {

                // TODO motor
                // TODO sensor
                // TODO vis (again)

                // TODO use timeout_logic
                let engine_response = futures::executor::block_on(burst_engine.execute_phase(phase, burst_index));
                match engine_response {
                    Ok(burst_output) => {
                        let worker_response = match burst_output {
                            BurstPhaseOutput::NoFurtherActionNeeded => {
                                // Inform worker that we are fine
                                BurstEngineWorkerResponse::NoFurtherActionNeeded
                            }
                            BurstPhaseOutput::MoreAllocationNeeded(allocations) => {
                                // this isnt a final message, just send and continue
                                BurstEngineWorkerResponse::MoreAllocationNeeded(allocations)
                            }
                            BurstPhaseOutput::BrainDeathTriggered => {
                                BurstEngineWorkerResponse::BrainDeathTriggered // not an error but the next iteration will have the worker get closed by the npu
                            }
                        };
                        // We should be able to do this since the pool immediately takes all messages from this queue before calling again
                        let pool_send_result = response_transmitter.try_send(worker_response);
                        if pool_send_result.is_ok() {
                            continue;
                        }
                        Some(pool_send_result.unwrap_err().into())
                    }
                    Err(e) => Some(e.into()),
                }
            }
            BurstEngineWorkerCommand::BurstIndexRollback { burst_index } => {
                panic!("not implemented!")
            }
            BurstEngineWorkerCommand::CommitSudoku => {
                // type kill in console right now
                // TODO should we tell the engine itself to stop? If so, here is the spot
                None // no errors to report while closing!
            }
        };

        // At this point, we are exiting the loop
        break BurstEngineWorkerConclusion::new(burst_engine, option_error);
    };
    // loop exited, We have our conclusion, return with it
    conclusion
}
