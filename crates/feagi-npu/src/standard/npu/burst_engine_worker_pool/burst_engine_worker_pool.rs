use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_models::wrapped_indexes::BurstIndex;
use crate::standard::npu::burst_engine::burst_engine_communication::KernelCommand;
use crate::standard::npu::burst_engine_worker::burst_engine_worker_communication::{
    BurstEngineWorkerCommand, BurstEngineWorkerResponse,
};
use crate::standard::npu::burst_engine_worker_pool::burst_engine_worker_pool_channels::BurstEngineWorkerPoolChannels;
use crate::standard::npu::npu_communication::NPUCommand;
use crate::standard::npu::npu_target_frequency::NPUTargetFrequency;

/// Runs the burst engine worker pool loop on its own thread.
pub fn burst_engine_worker_pool<FIQ: FeagiIndexQuantization + Send + 'static>(
    mut frequency: NPUTargetFrequency,
    channels: BurstEngineWorkerPoolChannels<FIQ>,
) {
    // TODO any init

    let mut current_burst_index: BurstIndex<FIQ::GlobalBurstIndexQuant> =
        BurstIndex::QUANT_ZERO; // TODO This should actually start at max / 2!
    let mut paused = false;
    let mut next_burst_at = std::time::Instant::now();

    loop {
        // Drain any wrapper commands that arrived since the last iteration.
        loop {
            match channels.npu_command_rx.try_recv() {
                Ok(NPUCommand::UpdateFrequency(new_frequency)) => {
                    frequency = new_frequency;
                }
                Ok(NPUCommand::Pause) => {
                    paused = true;
                    // TODO differentiate pause vs shutdown; add explicit resume command
                }
                Err(flume::TryRecvError::Empty) => break,
                Err(flume::TryRecvError::Disconnected) => {
                    // Wrapper gone; shut everything down cleanly.
                    stop_all_workers(&channels);
                    return;
                }
            }
        }

        if paused {
            // Modest sleep so we're not spinning while paused; the wrapper channel is
            // still drained on the next iteration so resume/shutdown can arrive.
            std::thread::sleep(std::time::Duration::from_millis(1));
            continue;
        }

        // Pace to the next burst boundary. If a previous burst overran the budget,
        // this sleep is zero-length and we execute back-to-back until caught up.
        let now = std::time::Instant::now();
        if now < next_burst_at {
            std::thread::sleep(next_burst_at - now);
        }
        next_burst_at += frequency.duration_between_bursts();

        // Broadcast the start of this burst to every worker.
        for pool_side in &channels.worker_channels {
            // TODO per-worker kernel selection; hardcoded to FullBurst as a placeholder
            let cmd = BurstEngineWorkerCommand::RunKernel {
                burst_index: current_burst_index,
                kernel: KernelCommand::FullBurst,
            };
            if pool_side.command_tx.send(cmd).is_err() {
                // TODO worker gone, something went wrong and we should probably stop and report this
            }
        }

        // Wait for each worker's response for this burst. Order-preserving iteration
        // by worker index; we accept whichever variant they return.
        for pool_side in &channels.worker_channels {
            match pool_side.response_rx.recv() {
                Ok(BurstEngineWorkerResponse::KernelRan { burst_index: _ }) => {
                    // TODO verify burst_index == current_burst_index
                }
                Ok(_) => {
                    // TODO unexpected response variant for a burst tick
                }
                Err(_) => {
                    // TODO worker gone, something is wrong
                }
            }
        }

        current_burst_index += BurstIndex::QUANT_ONE;
    }
}

/// Best-effort broadcast of `Stop` to every worker. Failures are ignored (a dead worker
/// cannot be told to stop).
fn stop_all_workers<FIQ: FeagiIndexQuantization>(
    channels: &BurstEngineWorkerPoolChannels<FIQ>,
) {
    for pool_side in &channels.worker_channels {
        let _ = pool_side.command_tx.send(BurstEngineWorkerCommand::Stop);
    }
    // TODO drain `Stopped` acknowledgements with a timeout
}
