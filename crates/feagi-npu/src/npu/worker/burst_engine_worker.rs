use std::time::Duration;
use crate::npu::worker::command_and_response::{BurstEngineWorkerCommand, BurstEngineWorkerResponse};
use feagi_data::channels::channels::{ChannelPair, ChannelReceivingError};
use feagi_data::channels::channels_flume::{FlumeChannelPair, InnerFlumeChannelPair, OuterFlumeChannelPair};
use feagi_models::quantization_levels::burst_engine_index_quantization::BurstEngineIndexQuantizationQuantizationNormal;
use feagi_models::quantization_levels::neuron_processing_unit_index_quantization::NeuronProcessingUnitIndexQuantization;
use feagi_npu_burst_engines::feagi_npu_burst_core::burst_engine_definitions::burst_engine::BurstEngine;
use feagi_npu_burst_engines::BurstEngineEnum;
use feagi_npu_burst_engines::feagi_npu_burst_core::burst_engine_definitions::burst_phase_output::BurstPhaseOutput;
use feagi_npu_burst_engines::feagi_npu_burst_core::burst_engine_definitions::burst_phases::RunBurstPhase;
use feagi_npu_burst_engines::feagi_npu_burst_core::errors::BurstEngineError;

/// Channel for sending commands to a burst engine worker
pub type BurstEngineWorkerCommander<NPUIQ: NeuronProcessingUnitIndexQuantization> =
    OuterFlumeChannelPair<BurstEngineWorkerCommand<NPUIQ>, BurstEngineWorkerResponse<NPUIQ>>;


/// Creates commander and follower channel pairs
pub fn make_commander_follower<NPUIQ: NeuronProcessingUnitIndexQuantization>(
    input_buffer_size: usize,
    output_buffer_size: usize,
) -> (
    BurstEngineWorkerCommander<NPUIQ>,
    InnerFlumeChannelPair<BurstEngineWorkerResponse<NPUIQ>, BurstEngineWorkerCommand<NPUIQ>>,
) {
    FlumeChannelPair::new_pairs(input_buffer_size, output_buffer_size)
}


pub fn burst_engine_worker<NPUIQ: NeuronProcessingUnitIndexQuantization>(
    mut burst_engine: BurstEngineEnum<NPUIQ, BurstEngineIndexQuantizationQuantizationNormal>, // TODO BEIQ quant enum!
    mut follower_channels: InnerFlumeChannelPair<BurstEngineWorkerResponse<NPUIQ>, BurstEngineWorkerCommand<NPUIQ>>,
    timeout_time: Duration
) {

    loop {
        // Block thread and wait for incoming command
        let command = match follower_channels.block_receive() {
            Ok(command) => command,
            Err(ChannelReceivingError::ReceiveFailed(e)) => {
                _ = follower_channels.try_send(BurstEngineWorkerResponse::Crashed("Burst Engine Channel Failed to Receive!"));
                break;
            }
            Err(ChannelReceivingError::ReceiveTimeout(e)) => {
                // We shouldn't be using timeouts here? This should be impossible
                panic!("Burst Engine Channel Failed to Receive before Timeout!");
            }
        };

        match command {
            BurstEngineWorkerCommand::RunPhases { burst_index, phase } => {
                let res = futures::executor::block_on(burst_engine.execute_phase(phase));

            }
            BurstEngineWorkerCommand::BurstIndexRollback { burst_index } => {
                panic!("not implemented!")
            }
            BurstEngineWorkerCommand::CommitSudoku => {
                // type kill in console right now
                break;
            }
        }
    }

    // loop exited, announce we are closing. Then killbind
    _ = follower_channels.try_send(BurstEngineWorkerResponse::Stopped);
}

fn run_burst_engine_phases<NPUIQ: NeuronProcessingUnitIndexQuantization>(
    burst_engine: &mut BurstEngineEnum<NPUIQ, BurstEngineIndexQuantizationQuantizationNormal>,
    phase: RunBurstPhase,
    timeout: Duration)
    -> Result<BurstPhaseOutput, BurstEngineError>
{
    // TODO timeout for this call specifically?
    let res = futures::executor::block_on(burst_engine.execute_phase(phase));

    if let Err(e) = res {
        match e {
            BurstEngineError::NPUEtc(e) => {

            }
            BurstEngineError::Phase(e) => {}
            BurstEngineError::DataCorruption(e) => {}
        }
    }

    match res {
        Ok(_) => {}
        Err(_) => {}
    }

}

// TODO maybe make crash errors their own sub enum?

/// closes the worker by first sending the optional request (attempting to be error), then closing. Handles timeouts as best as possible
fn close_worker<NPUIQ: NeuronProcessingUnitIndexQuantization>(attempting_final_error: Option<BurstEngineWorkerResponse<NPUIQ>>) {

}
