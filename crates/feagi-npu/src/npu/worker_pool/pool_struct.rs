use std::ops::ControlFlow;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_models::wrapped_indexes::BurstIndex;
use crate::npu::npu_target_frequency::NPUTargetFrequency;
use crate::npu::worker::command_and_response::{BurstEngineWorkerCommand, BurstEngineWorkerResponse};
use crate::npu::worker_pool::command_and_response::{BurstEngineWorkerFeedback, BurstEngineWorkerPoolCommand};

/// Channel the pool uses to talk to the NPU: sends feedback, receives pool commands.
pub type PoolFeedbackChannel<FIQ: FeagiIndexQuantization> =
    InnerFlumeChannelPair<BurstEngineWorkerFeedback<FIQ>, BurstEngineWorkerPoolCommand<FIQ>>;

/// Channel the pool uses to drive a single worker: sends commands, receives responses.
type WorkerCommandChannel<FIQ: FeagiIndexQuantization> =
OuterFlumeChannelPair<BurstEngineWorkerCommand<FIQ>, BurstEngineWorkerResponse<FIQ>>;

/// Runtime state for a burst engine worker pool
pub struct BurstEngineWorkerPool<FIQ: FeagiIndexQuantization> {
    feedback_channels: PoolFeedbackChannel<FIQ>,
    worker_commanders: Vec<WorkerCommandChannel<FIQ>>,
    frequency: NPUTargetFrequency,
    bursts_paused: bool,
    burst_index: BurstIndex<FIQ::BurstIndexQuant>,
}

impl<FIQ: FeagiIndexQuantization> BurstEngineWorkerPool<FIQ> {
    
    /// Create a new instance
    pub fn new(
        feedback_channels: PoolFeedbackChannel<FIQ>,
        worker_commanders: Vec<WorkerCommandChannel<FIQ>>,
        initial_frequency: NPUTargetFrequency,
    ) -> Self {
        Self {
            feedback_channels,
            worker_commanders,
            frequency: initial_frequency,
            bursts_paused: false,
            burst_index: BurstIndex::QUANT_MAX / (BurstIndex::QUANT_ONE + BurstIndex::QUANT_ONE),
        }
    }

    /// Drive the pool until an iteration signals a stop, then tear the workers down.
    pub fn run(&mut self) {
        loop {
            // Run iteration, get if we should break or not
            let control_flow = self.run_iteration();
            
            if control_flow.is_break() {
                break;
            }
        }
        // Always terminate the workers on the way out, regardless of why we stopped,
        self.shutdown_workers();
    }

    /// One pass of the control loop. An early return here plays the role the old `continue` did.
    fn run_iteration(&mut self) -> ControlFlow<()> {
        
        // Get next step enum depending on if/what we receive as an incoming command
        let next_step = self.receive_next_command();
        
        match next_step {
            PoolNextStep::HandleIncomingCommand(command) => {
                
                let result_flow = self.handle_npu_command(command);
                result_flow
            },
            PoolNextStep::RunBurst => {
                let result_flow = self.run_burst();
                result_flow
            },
            PoolNextStep::Idle => ControlFlow::Continue(()),
            PoolNextStep::Stop => ControlFlow::Break(()),
        }
    }

    /// Obtain the next thing to do. Paused blocks for a command; unpaused polls, treating "nothing
    /// pending" as permission to run a burst this iteration (even though that shouldnt happen?)
    fn receive_next_command(&mut self) -> PoolNextStep<FIQ> {
        
        // If paused, we block the thread for the next command, otherwise we just try quickly
        let received = if self.bursts_paused {
            self.feedback_channels.block_receive().map(Some)
        } else {
            self.feedback_channels.try_receive()
        };

        match received {
            
            Ok(Some(command)) => PoolNextStep::HandleIncomingCommand(command),
            // Only reachable while unpaused: no command waiting, so proceed to a burst.
            Ok(None) => PoolNextStep::RunBurst,
            // A timeout is not expected on block/try receive; skip this iteration and retry.
            Err(ChannelReceivingError::ReceiveTimeout(_)) => PoolNextStep::Idle,
            Err(ChannelReceivingError::ReceiveFailed(_)) => {
                _ = self.feedback_channels.try_send(BurstEngineWorkerFeedback::UnknownFailure("Pool receiver failure"));
                PoolNextStep::Stop
            }
        }
    }

    /// Apply a command from the NPU. Returns `Break` when the command (or a fatal error handling it)
    /// should stop the pool.
    fn handle_npu_command(&mut self, command: BurstEngineWorkerPoolCommand<FIQ>) -> ControlFlow<()> {
        match command {
            BurstEngineWorkerPoolCommand::SpecificEngineCommand { burst_engine_index, command } => {
                let Some(commander) = self.worker_commanders.get_mut(burst_engine_index as usize)
                else {
                    // The NPU referenced a worker that does not exist
                    _ = self.feedback_channels.try_send(BurstEngineWorkerFeedback::UnknownFailure("Invalid Burst Engine Index Sent"));
                    return ControlFlow::Break(());
                };

                // A failed send or receive to a worker means that worker is probably crashed
                if commander.block_send(command).is_err() {
                    _ = self.feedback_channels.try_send(BurstEngineWorkerFeedback::WorkerCrashed(burst_engine_index, "Failed to send command to worker"));
                    return ControlFlow::Break(());
                }

                // Wait for worker to finish // TODO timeout
                let response = match commander.block_receive() {
                    Ok(response) => response,
                    Err(_) => {
                        _ = self.feedback_channels.try_send(BurstEngineWorkerFeedback::WorkerCrashed(burst_engine_index, "Failed to receive response from worker"));
                        return ControlFlow::Break(());
                    }
                };

                Self::handle_worker_response(&mut self.feedback_channels, burst_engine_index, response)
            }
            BurstEngineWorkerPoolCommand::Pause => {
                self.bursts_paused = true;
                ControlFlow::Continue(())
            }
            BurstEngineWorkerPoolCommand::SetBurstFrequency(frequency) => {
                self.bursts_paused = false;
                self.frequency = frequency;
                ControlFlow::Continue(())
            }
            BurstEngineWorkerPoolCommand::StopAllWorkersAndClose => {
                // Graceful shutdown requested: acknowledge it, then let `run` terminate the workers.
                _ = self.feedback_channels.try_send(BurstEngineWorkerFeedback::SafelyStopped);
                ControlFlow::Break(())
            }
        }
    }

    /// Fan a burst out to every worker, then fan the responses back in. Any failed dispatch or
    /// response is treated as a worker crash and stops the pool.
    fn run_burst(&mut self) -> ControlFlow<()> {
        // TODO increment burst index
        let _current_time = std::time::Instant::now();

        let burst_index = self.burst_index;
        for (index, worker_commander) in self.worker_commanders.iter_mut().enumerate() {
            if worker_commander.try_send(BurstEngineWorkerCommand::RunPhases {
                burst_index,
                phase: Default::default(),
            }).is_err() {
                _ = self.feedback_channels.try_send(BurstEngineWorkerFeedback::WorkerCrashed(index as u16, "Failed to dispatch burst to worker"));
                return ControlFlow::Break(());
            }
        }

        // Block until every worker has finished this burst so all engines stay in lockstep.
        for index in 0..self.worker_commanders.len() {
            let response = match self.worker_commanders[index].block_receive() {
                Ok(response) => response,
                Err(_) => {
                    _ = self.feedback_channels.try_send(BurstEngineWorkerFeedback::WorkerCrashed(index as u16, "Failed to receive burst completion from worker"));
                    return ControlFlow::Break(());
                }
            };

            if Self::handle_worker_response(&mut self.feedback_channels, index as u16, response).is_break() {
                return ControlFlow::Break(());
            }
        }

        ControlFlow::Continue(())
        // TODO thread sleep close to time to loop and then spin loop till final time to maintain freq
    }

    /// Interpret a worker's response. Normal completions continue the loop.
    /// Crashes and brain death are forwarded to the NPU and the pool is stopped
    fn handle_worker_response(
        feedback_channels: &mut PoolFeedbackChannel<FIQ>,
        burst_engine_index: u16,
        response: BurstEngineWorkerResponse<FIQ>,
    ) -> ControlFlow<()> {
        match response {
            BurstEngineWorkerResponse::KernelRan { .. } => ControlFlow::Continue(()),
            // A worker acknowledging its own stop is expected during teardown, not an error.
            BurstEngineWorkerResponse::Stopped => ControlFlow::Continue(()),
            BurstEngineWorkerResponse::BrainDeathTriggered => {
                _ = feedback_channels.try_send(BurstEngineWorkerFeedback::BrainDeathTriggered);
                ControlFlow::Break(())
            }
            BurstEngineWorkerResponse::Crashed(reason) => {
                _ = feedback_channels.try_send(BurstEngineWorkerFeedback::WorkerCrashed(burst_engine_index, reason));
                ControlFlow::Break(())
            }
        }
    }

    /// Tell every worker to terminate. Failures are reported but do not stop us from asking the rest
    /// to shut down.
    fn shutdown_workers(&mut self) {
        for index in 0..self.worker_commanders.len() {
            if self.worker_commanders[index].block_send(BurstEngineWorkerCommand::CommitSudoku).is_err() {
                _ = self.feedback_channels.try_send(BurstEngineWorkerFeedback::WorkerCrashed(index as u16, "Failed to send stop command to worker during shutdown"));
            }
        }
    }
}

/// Intent of the pool
enum PoolNextStep<FIQ: FeagiIndexQuantization> {
    /// A command from the NPU is ready to process.
    HandleIncomingCommand(BurstEngineWorkerPoolCommand<FIQ>),
    /// Nothing pending while unpaused: run a burst.
    RunBurst,
    /// Do nothing this iteration and loop again.
    Idle,
    /// Stop the pool.
    Stop,
}