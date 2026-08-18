use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::standard::npu::burst_engine::burst_engine::SyncBurstEngine;
use crate::standard::npu::burst_engine::burst_engine_enum::BurstEngineEnum;
use crate::standard::npu::burst_engine_worker::burst_engine_worker_channels::BurstEngineWorkerChannels;
use crate::standard::npu::burst_engine_worker::burst_engine_worker_communication::{
    BurstEngineWorkerCommand, BurstEngineWorkerResponse,
};

/// Runs a burst engine loop on its own thread.
///
/// Blocks on the command channel each iteration. A closed command channel (all pool-side
/// senders dropped) is treated as an implicit stop and the loop exits cleanly.
///
/// Intended usage:
/// ```ignore
/// let (pool_side, worker_side) = BurstEngineWorkerChannels::<FIQ>::new_pair(2);
/// std::thread::spawn(move || burst_engine_worker(burst_engine, worker_side));
/// // ... pool_side goes to the pool struct ...
/// ```
pub fn burst_engine_worker<FIQ: FeagiIndexQuantization + Send + 'static>(
    mut burst_engine: BurstEngineEnum<FIQ>,
    channels: BurstEngineWorkerChannels<FIQ>,
) {
    // TODO any init
    
    loop {
        let command = match channels.command_rx.recv() {
            Ok(c) => c,
            Err(_) => break, // all senders dropped -> implicit stop
        };
        
        match command {
            BurstEngineWorkerCommand::RunKernel { burst_index, kernel } => {
                // TODO propagate kernel error via the response payload
                let _ = burst_engine.run_kernel(kernel);
                
                if channels
                    .response_tx
                    .send(BurstEngineWorkerResponse::KernelRan { burst_index })
                    .is_err()
                {
                    // Pool receiver gone; nothing more we can do.
                    break;
                }
            }
            BurstEngineWorkerCommand::EngineConnectomeEdits(edits) => {
                let mut responses = Vec::with_capacity(edits.len());
                for edit in edits {
                    // TODO propagate edit errors
                    if let Ok(response) = burst_engine.edit_connectome(edit) {
                        responses.push(response);
                    }
                }
                if channels
                    .response_tx
                    .send(BurstEngineWorkerResponse::EngineConnectomeEdited(responses))
                    .is_err()
                {
                    break;
                }
            }
            BurstEngineWorkerCommand::Stop => {
                // Best-effort acknowledgement; pool may already be gone.
                let _ = channels
                    .response_tx
                    .send(BurstEngineWorkerResponse::Stopped);
                break;
            }
        }
    }
    
    // TODO optionally return `burst_engine` (via oneshot embedded in Stop) so callers can
    //      recover its state.
}
