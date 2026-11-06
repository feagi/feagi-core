//! The worker's side of its command channel. `FeagiWorkerCommandRouter` owns the worker's
//! `FeagiWorkerCommandProcessor` and handles universal commands (`Stop`). A worker can check for
//! commands between other work with `try_handle_next`, or sit idle until one arrives with
//! `handle_next`, switching between the two as its state changes

use core::ops::ControlFlow;

use crate::parallelism::feagi_channel::base_channels::base_errors::FeagiChannelReceiveError;
use crate::parallelism::feagi_channel::request_response_channel::FeagiRequestResponseChannelBackend;
use crate::parallelism::feagi_channel::request_response_channel::FeagiResponder;
use crate::parallelism::feagi_worker::worker_internals::feagi_worker_command_processor::FeagiWorkerCommandProcessor;
use crate::parallelism::feagi_worker::worker_commands::{FeagiWorkerRequest, FeagiWorkerResponse};

/// The responder half of the pair whose requester is given to `FeagiWorker::new`
pub type FeagiWorkerCommandResponder<ChannelBackend, CommandProcessor> = FeagiResponder<
    FeagiWorkerRequest<<CommandProcessor as FeagiWorkerCommandProcessor>::CustomRequest>,
    FeagiWorkerResponse<<CommandProcessor as FeagiWorkerCommandProcessor>::CustomResponse>,
    ChannelBackend,
>;

/// Why a worker should leave its loop
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeagiWorkerExit {
    /// `Stop` was handled and acknowledged
    Stopped,
    /// The `FeagiWorker` was dropped, so nothing can command or stop this worker anymore. The
    /// worker should kill-bind
    Disconnected,
}

/// The worker's side of the command channel. Do NOT allow this to be clonable! There is exactly
/// one per command channel
pub struct FeagiWorkerCommandRouter<ChannelBackend, CommandProcessor>
where
    ChannelBackend: FeagiRequestResponseChannelBackend,
    CommandProcessor: FeagiWorkerCommandProcessor,
{
    processor: CommandProcessor,
    responder: FeagiWorkerCommandResponder<ChannelBackend, CommandProcessor>,
}

impl<ChannelBackend, CommandProcessor> FeagiWorkerCommandRouter<ChannelBackend, CommandProcessor>
where
    ChannelBackend: FeagiRequestResponseChannelBackend,
    CommandProcessor: FeagiWorkerCommandProcessor,
{
    /// Routes commands from `responder` through `processor`
    pub fn new(processor: CommandProcessor, responder: FeagiWorkerCommandResponder<ChannelBackend, CommandProcessor>) -> Self {
        FeagiWorkerCommandRouter { processor, responder }
    }

    /// Handles one command if one is queued, never waits. Continues with whether a command was
    /// handled, or breaks with why the worker should exit
    pub async fn try_handle_next(&mut self) -> ControlFlow<FeagiWorkerExit, bool> {
        let mut stopped = false;
        let result = self.responder.try_process_next(Self::route(&mut self.processor, &mut stopped)).await;
        if let Err(FeagiChannelReceiveError::ChannelEmpty) = result {
            return ControlFlow::Continue(false);
        }
        Self::exit_from_result(result, stopped)?;
        ControlFlow::Continue(true)
    }

    /// Waits for the next command, then handles it. Breaks with why the worker should exit
    ///
    /// This future can be raced against other work (as in a `select`), but only against branches
    /// that resolve rapidly. If another branch wins while a command is being handled, the
    /// handling is cut short: the `FeagiWorker` receives `ResponderDroppedRequest`, and a cut
    /// short `on_stop` never finishes its cleanup
    pub async fn await_handle_next(&mut self) -> ControlFlow<FeagiWorkerExit> {
        let mut stopped = false;
        let result = self.responder.async_process_next(Self::route(&mut self.processor, &mut stopped)).await;
        Self::exit_from_result(result, stopped)
    }

    /// Waits for and handles commands until the worker is stopped or its `FeagiWorker` is
    /// dropped. Gives back the worker's state, for any cleanup after exiting, along with why it
    /// exited
    pub async fn run_until_stopped(mut self) -> (CommandProcessor, FeagiWorkerExit) {
        loop {
            if let ControlFlow::Break(exit) = self.await_handle_next().await {
                return (self.processor, exit);
            }
        }
    }

    /// The worker's state, for the work it does between commands
    pub fn processor_mut(&mut self) -> &mut CommandProcessor {
        &mut self.processor
    }

    /// Gives back the worker's state, for any cleanup once the worker's loop has exited
    pub fn into_processor(self) -> CommandProcessor {
        self.processor
    }

    /// Dispatches a command to `processor` and wraps its result in the matching reply variant.
    /// Takes the processor rather than `self` so the responder can be borrowed alongside it
    fn route<'a>(
        processor: &'a mut CommandProcessor,
        stopped: &'a mut bool,
    ) -> impl AsyncFnOnce(FeagiWorkerRequest<CommandProcessor::CustomRequest>) -> FeagiWorkerResponse<CommandProcessor::CustomResponse> + 'a {
        async move |request| match request {
            FeagiWorkerRequest::Stop => {
                processor.on_stop().await;
                *stopped = true;
                FeagiWorkerResponse::Acknowledged
            }
            FeagiWorkerRequest::Custom(request) => FeagiWorkerResponse::Custom(processor.on_custom(request).await),
            // Never sent by `FeagiWorker`, only the value of an unused slot
            FeagiWorkerRequest::Nothing => FeagiWorkerResponse::Nothing,
        }
    }

    /// Maps a processed (or failed) receive to whether the worker should keep running. Any
    /// receive error here means the `FeagiWorker` is gone
    fn exit_from_result(result: Result<(), FeagiChannelReceiveError>, stopped: bool) -> ControlFlow<FeagiWorkerExit> {
        match result {
            Ok(()) if stopped => ControlFlow::Break(FeagiWorkerExit::Stopped),
            Ok(()) => ControlFlow::Continue(()),
            Err(_) => ControlFlow::Break(FeagiWorkerExit::Disconnected),
        }
    }
}
