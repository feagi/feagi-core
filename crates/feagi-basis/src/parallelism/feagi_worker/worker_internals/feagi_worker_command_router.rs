//! The worker's side of its command channel. Two routers cover the two shapes a worker takes:
//! `FeagiWorkerPollingRouter` for workers that loop continuously and check for commands between
//! other work, and `FeagiWorkerAwaitingRouter` for workers that sit idle until a command arrives.
//! Both own the worker's `FeagiWorkerCommandProcessor` and handle universal commands (`Stop`) the
//! same way

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

//region Polling Router

/// The worker's side of the command channel, for workers that loop continuously. Each pass of the
/// worker's loop calls `try_handle_next`, which never waits. Do NOT allow this to be clonable!
/// There is exactly one per command channel
pub struct FeagiWorkerPollingRouter<ChannelBackend, CommandProcessor>
where
    ChannelBackend: FeagiRequestResponseChannelBackend,
    CommandProcessor: FeagiWorkerCommandProcessor,
{
    core: FeagiWorkerCommandCore<ChannelBackend, CommandProcessor>,
}

impl<ChannelBackend, CommandProcessor> FeagiWorkerPollingRouter<ChannelBackend, CommandProcessor>
where
    ChannelBackend: FeagiRequestResponseChannelBackend,
    CommandProcessor: FeagiWorkerCommandProcessor,
{
    /// Routes commands from `responder` through `processor`
    pub fn new(processor: CommandProcessor, responder: FeagiWorkerCommandResponder<ChannelBackend, CommandProcessor>) -> Self {
        FeagiWorkerPollingRouter {
            core: FeagiWorkerCommandCore { processor, responder },
        }
    }

    /// Handles one command if one is queued, otherwise returns `Idle` immediately
    pub async fn try_handle_next(&mut self) -> FeagiWorkerStep {
        self.core.try_handle_next().await
    }

    /// The worker's state, for the work it does between commands
    pub fn processor_mut(&mut self) -> &mut CommandProcessor {
        &mut self.core.processor
    }

    /// Gives back the worker's state, for any cleanup once the worker's loop has exited
    pub fn into_processor(self) -> CommandProcessor {
        self.core.processor
    }
}

/// What happened when a polling worker checked its command channel
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeagiWorkerStep {
    /// A command was handled, keep running
    Handled,
    /// No command was waiting, keep running
    Idle,
    /// `Stop` was handled and acknowledged. The worker should exit its loop
    Stopped,
    /// The `FeagiWorker` was dropped, so nothing can command or stop this worker anymore. The
    /// worker should kill-bind
    Disconnected,
}

//endregion

//region Awaiting Router

/// The worker's side of the command channel, for workers that are idle until a command arrives.
/// Do NOT allow this to be clonable! There is exactly one per command channel
pub struct FeagiWorkerAwaitingRouter<ChannelBackend, CommandProcessor>
where
    ChannelBackend: FeagiRequestResponseChannelBackend,
    CommandProcessor: FeagiWorkerCommandProcessor,
{
    core: FeagiWorkerCommandCore<ChannelBackend, CommandProcessor>,
}

impl<ChannelBackend, CommandProcessor> FeagiWorkerAwaitingRouter<ChannelBackend, CommandProcessor>
where
    ChannelBackend: FeagiRequestResponseChannelBackend,
    CommandProcessor: FeagiWorkerCommandProcessor,
{
    /// Routes commands from `responder` through `processor`
    pub fn new(processor: CommandProcessor, responder: FeagiWorkerCommandResponder<ChannelBackend, CommandProcessor>) -> Self {
        FeagiWorkerAwaitingRouter {
            core: FeagiWorkerCommandCore { processor, responder },
        }
    }

    /// Handles commands until the worker is stopped or its `FeagiWorker` is dropped. Gives back
    /// the worker's state, for any cleanup after exiting, along with why it exited
    pub async fn run_until_stopped(mut self) -> (CommandProcessor, FeagiWorkerExit) {
        loop {
            if let Some(exit) = self.handle_next().await {
                return (self.core.processor, exit);
            }
        }
    }

    /// Waits for the next command, then handles it. Returns `None` if the worker should keep
    /// running
    ///
    /// This future can be raced against other work (as in a `select`), but only against branches
    /// that resolve rapidly. If another branch wins while a command is being handled, the
    /// handling is cut short: the `FeagiWorker` receives `ResponderDroppedRequest`, and a cut
    /// short `on_stop` never finishes its cleanup
    pub async fn handle_next(&mut self) -> Option<FeagiWorkerExit> {
        match self.core.async_handle_next().await {
            FeagiWorkerStep::Handled | FeagiWorkerStep::Idle => None,
            FeagiWorkerStep::Stopped => Some(FeagiWorkerExit::Stopped),
            FeagiWorkerStep::Disconnected => Some(FeagiWorkerExit::Disconnected),
        }
    }

    /// The worker's state, for the work it does between commands
    pub fn processor_mut(&mut self) -> &mut CommandProcessor {
        &mut self.core.processor
    }
}

/// Why an awaiting worker stopped handling commands
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeagiWorkerExit {
    /// `Stop` was handled and acknowledged
    Stopped,
    /// The `FeagiWorker` was dropped, so nothing can command or stop this worker anymore
    Disconnected,
}

//endregion

//region Shared Core

/// What both routers share: the worker's state and the command channel, and the handling of
/// universal commands
struct FeagiWorkerCommandCore<ChannelBackend, CommandProcessor>
where
    ChannelBackend: FeagiRequestResponseChannelBackend,
    CommandProcessor: FeagiWorkerCommandProcessor,
{
    processor: CommandProcessor,
    responder: FeagiWorkerCommandResponder<ChannelBackend, CommandProcessor>,
}

impl<ChannelBackend, CommandProcessor> FeagiWorkerCommandCore<ChannelBackend, CommandProcessor>
where
    ChannelBackend: FeagiRequestResponseChannelBackend,
    CommandProcessor: FeagiWorkerCommandProcessor,
{
    /// Handles one command if one is queued, otherwise returns `Idle` immediately
    async fn try_handle_next(&mut self) -> FeagiWorkerStep {
        let mut step = FeagiWorkerStep::Handled;
        let result = self.responder.try_process_next(Self::route(&mut self.processor, &mut step)).await;
        Self::step_from_result(result, step)
    }

    /// Waits for the next command, then handles it. Never returns `Idle`
    async fn async_handle_next(&mut self) -> FeagiWorkerStep {
        let mut step = FeagiWorkerStep::Handled;
        let result = self.responder.async_process_next(Self::route(&mut self.processor, &mut step)).await;
        Self::step_from_result(result, step)
    }

    /// Dispatches a command to `processor` and wraps its result in the matching reply variant.
    /// Takes the processor rather than `self` so the responder can be borrowed alongside it
    fn route<'a>(
        processor: &'a mut CommandProcessor,
        step: &'a mut FeagiWorkerStep,
    ) -> impl AsyncFnOnce(FeagiWorkerRequest<CommandProcessor::CustomRequest>) -> FeagiWorkerResponse<CommandProcessor::CustomResponse> + 'a {
        async move |request| match request {
            FeagiWorkerRequest::Stop => {
                processor.on_stop().await;
                *step = FeagiWorkerStep::Stopped;
                FeagiWorkerResponse::Acknowledged
            }
            FeagiWorkerRequest::Custom(request) => FeagiWorkerResponse::Custom(processor.on_custom(request).await),
            // Never sent by `FeagiWorker`, only the value of an unused slot
            FeagiWorkerRequest::Nothing => FeagiWorkerResponse::Nothing,
        }
    }

    fn step_from_result(result: Result<(), FeagiChannelReceiveError>, step: FeagiWorkerStep) -> FeagiWorkerStep {
        match result {
            Ok(()) => step,
            Err(FeagiChannelReceiveError::ChannelEmpty) => FeagiWorkerStep::Idle,
            Err(_) => FeagiWorkerStep::Disconnected,
        }
    }
}

//endregion
