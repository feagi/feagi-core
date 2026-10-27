//! The worker's side of its command channel. Handles one command at a time, so each worker keeps
//! its own loop and can interleave commands with whatever else it does

use crate::parallelism::feagi_channel::base_channels::base_errors::FeagiChannelReceiveError;
use crate::parallelism::feagi_channel::request_response_channel::FeagiRequestResponseChannelBackend;
use crate::parallelism::feagi_channel::request_response_channel::FeagiResponder;
use crate::parallelism::feagi_worker::worker_internals::feagi_worker_command_processor::FeagiWorkerCommandProcessor;
use crate::parallelism::feagi_worker::worker_internals::worker_terminal::worker_commands::{FeagiWorkerRequest, FeagiWorkerResponse};

/// The worker's side of the command channel from its `FeagiWorker`. Routes commands through a 
/// `FeagiWorkerCommandProcessor`. Do NOT allow this to be clonable! There is exactly one per-command channel
pub struct FeagiWorkerCommandRouter<ChannelBackend, CommandProcessor>
where
    ChannelBackend: FeagiRequestResponseChannelBackend,
    CommandProcessor: FeagiWorkerCommandProcessor
{
    processor: CommandProcessor,
    responder: FeagiResponder<FeagiWorkerRequest<CommandProcessor::CustomRequest>, FeagiWorkerResponse<CommandProcessor::CustomResponse>, ChannelBackend>,
}

impl<ChannelBackend, CommandProcessor> FeagiWorkerCommandRouter<ChannelBackend, CommandProcessor>
where
    ChannelBackend: FeagiRequestResponseChannelBackend,
    CommandProcessor: FeagiWorkerCommandProcessor
{
    /// Wraps the responder half of the pair whose requester is given to `FeagiWorker::new`
    pub fn new(processor: CommandProcessor, responder: FeagiResponder<FeagiWorkerRequest<CommandProcessor::CustomRequest>, FeagiWorkerResponse<CommandProcessor::CustomResponse>, ChannelBackend>) -> Self {
        FeagiWorkerCommandRouter {
            processor,
            responder 
        }
    }

    /// Handles one command if one is queued, otherwise returns `Idle` immediately
    pub async fn try_handle_next(&mut self) -> FeagiWorkerStep
    {
        let mut step = FeagiWorkerStep::Handled;
        let result = self.responder.try_process_next(Self.command_processor(&mut step)).await;
        Self::step_from_result(result, step)
    }

    /// Waits for the next command, then handles it
    ///
    /// This future can be raced against other work (as in a `select`), but only against branches
    /// that resolve rapidly. If another branch wins while a command is being handled, the
    /// handling is cut short: the `FeagiWorker` receives `ResponderDroppedRequest`, and a cut
    /// short `on_stop` never finishes its cleanup
    pub async fn async_handle_next(&mut self) -> FeagiWorkerStep
    {
        let mut step = FeagiWorkerStep::Handled;
        self.responder.
        
        
        let result = self.responder.async_process_next(Self.command_processor(&mut step)).await;
        Self::step_from_result(result, step)
    }

    /// Handles commands until the worker is stopped or its `FeagiWorker` is dropped. For workers
    /// that do nothing but answer commands
    pub async fn run_until_stopped<Handler>(&self, handler: &mut Handler)
    where
        Handler: FeagiWorkerCommandProcessor<CustomRequest = CustomRequest, CustomResponse = CustomResponse>,
    {
        loop {
            match self.async_handle_next(handler).await {
                FeagiWorkerStep::Stopped | FeagiWorkerStep::Disconnected => return,
                FeagiWorkerStep::Handled | FeagiWorkerStep::Idle => {}
            }
        }
    }

    /// Dispatches a command to `handler` and wraps its result in the matching reply variant
    fn command_processor<'a>(
        &'a mut self,
        step: &'a mut FeagiWorkerStep,
    ) -> impl AsyncFnOnce(FeagiWorkerRequest<CommandProcessor::CustomRequest>) -> FeagiWorkerResponse<CommandProcessor::CustomResponse> + 'a
    {
        async move |request| match request {
            FeagiWorkerRequest::Stop => {
                self.processor.on_stop().await;
                *step = FeagiWorkerStep::Stopped;
                FeagiWorkerResponse::Acknowledged
            }
            FeagiWorkerRequest::Custom(request) => FeagiWorkerResponse::Custom(self.processor.on_custom(request).await),
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

/// What happened when the worker checked its command channel
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
