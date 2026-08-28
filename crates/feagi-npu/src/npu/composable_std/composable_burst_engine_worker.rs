use feagi_data::channels::channels_flume::{FlumeChannelPair, InnerFlumeChannelPair, OuterFlumeChannelPair};
use feagi_data::channels::channels::{ChannelReceivingError, ChannelPair};
use feagi_models::quantization_levels::burst_engine_index_quantization::BurstEngineIndexQuantizationQuantizationNormal;
use feagi_models::quantization_levels::neuron_processing_unit_index_quantization::NeuronProcessingUnitIndexQuantization;
use feagi_models::wrapped_indexes::BurstIndex;
use feagi_npu_burst_engines::feagi_npu_burst_core::burst_engine_definitions::burst_engine::BurstEngine;
use feagi_npu_burst_engines::feagi_npu_burst_core::burst_engine_definitions::burst_phases::RunBurstPhase;
use feagi_npu_burst_engines::ComposableBurstEngineEnum;

pub type BurstEngineWorkerCommander<NPUIQ: NeuronProcessingUnitIndexQuantization> =
    OuterFlumeChannelPair<ComposableBurstEngineWorkerCommand<NPUIQ>, ComposableBurstEngineWorkerResponse<NPUIQ>>;

pub fn composable_burst_engine_worker<NPUIQ: NeuronProcessingUnitIndexQuantization>(
    mut burst_engine: ComposableBurstEngineEnum<NPUIQ, BurstEngineIndexQuantizationQuantizationNormal>, // TODO BEIQ quant enum!
    mut follower_channels: InnerFlumeChannelPair<ComposableBurstEngineWorkerResponse<NPUIQ>, ComposableBurstEngineWorkerCommand<NPUIQ>>,
) {
    // TODO Init

    loop {
        let command = follower_channels.block_receive();

        match command {
            Err(error) => {
                match error {
                    ChannelReceivingError::ReceiveFailed(e) => {
                        _ = follower_channels.try_send(ComposableBurstEngineWorkerResponse::Crashed("Burst Engine Channel Failed to Receive!"));
                        break;
                    }
                    ChannelReceivingError::ReceiveTimeout(e) => {
                        // We shouldn't be using timeouts here? This should be impossible
                        panic!("Burst Engine Channel Failed to Receive before Timeout!");
                    }
                }
            }
            Ok(command) => {
                match command {
                    ComposableBurstEngineWorkerCommand::RunPhases { burst_index, phase } => {
                        let res = burst_engine.execute_phase(phase).await;

                    }
                    ComposableBurstEngineWorkerCommand::BurstIndexRollback { burst_index } => {
                        panic!("not implemented!")
                    }
                    ComposableBurstEngineWorkerCommand::CommitSudoku => {
                        // type kill in console right now
                        break;
                    }
                }
            }
        }
    }

    // loop exited, announce we are closing. Then killbind
    _ = follower_channels.try_send(ComposableBurstEngineWorkerResponse::Stopped);

}

/// Creates commander and follower channel pairs
pub fn make_commander_follower<NPUIQ: NeuronProcessingUnitIndexQuantization>(
    input_buffer_size: usize,
    output_buffer_size: usize,
) -> (
    BurstEngineWorkerCommander<NPUIQ>,
    InnerFlumeChannelPair<ComposableBurstEngineWorkerResponse<NPUIQ>, ComposableBurstEngineWorkerCommand<NPUIQ>>,
) {
    FlumeChannelPair::new_pairs(input_buffer_size, output_buffer_size)
}

/// The commands to control what action a burst engine worker should take its next loop iteration
pub enum ComposableBurstEngineWorkerCommand<NPUIQ: NeuronProcessingUnitIndexQuantization> {
    /// Run the burst engine, either the default full burst or a specific phase.
    RunPhases {
        burst_index: BurstIndex<NPUIQ::BurstIndexQuant>,
        phase: RunBurstPhase,
    },
    /// Burst index is overflowing, its being changed now to this
    BurstIndexRollback { burst_index: BurstIndex<NPUIQ::BurstIndexQuant> },
    // TODO connectome edit
    /// Hard terminate this worker https://www.youtube.com/watch?v=x012BnKWi3g&t=97s
    CommitSudoku,
}

/// The responses that come from the Burst Engine Worker at the end of each burst
pub enum ComposableBurstEngineWorkerResponse<NPUIQ: NeuronProcessingUnitIndexQuantization> {
    KernelRan {
        burst_index: BurstIndex<NPUIQ::BurstIndexQuant>,
        // TODO carry Result<(), FeagiBurstEngineError> or metrics
    },
    //EngineConnectomeEdited(Vec<EngineConnectomeEditResponse<FIQ>>),
    Stopped,
    /// Hes dead, jim
    BrainDeathTriggered,
    /// While probably not the cause, lets blame Windows for this anyways
    Crashed(&'static str),
}
