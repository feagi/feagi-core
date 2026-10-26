//! The worker's side of its command channel. Handles one command at a time, so each worker keeps
//! its own loop and can interleave commands with whatever else it does

use crate::parallelism::feagi_channel::base_channels::base_errors::FeagiChannelReceiveError;
use crate::parallelism::feagi_channel::request_response_channel::FeagiRequestResponseChannelBackend;
use crate::parallelism::feagi_channel::request_response_channel::FeagiResponder;
use crate::parallelism::feagi_worker::worker_request_response::{FeagiWorkerRequest, FeagiWorkerResponse};

/// Implemented by a worker's state to answer the commands sent through its `FeagiWorker`. The
/// reply variant is chosen by `FeagiWorkerCommands`, so a command can never get the wrong kind
/// of reply
pub trait FeagiWorkerCommandHandler {
    /// What type of custom request going in is supported
    type CustomRequest;
    /// What type of custom response coming out is supported
    type CustomResponse;

    /// Called when the worker is told to stop. Runs before the stop is acknowledged, so
    /// `FeagiWorker::stop` returning means this has finished. Override this if custom behavior
    /// is needed on the given worker
    async fn on_stop(&mut self) {}

    /// Answers a custom request. Either replies right away (queries) or applies a state change
    /// and returns some form of acknowledgement. Do not block on here as it would impede the
    /// worker
    async fn on_custom(&mut self, request: Self::CustomRequest) -> Self::CustomResponse;
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

/// The worker's side of the command channel from its `FeagiWorker` Do NOT allow this to be
/// clonable! There is exactly one per-command channel
pub struct FeagiWorkerCommands<CustomRequest, CustomResponse, Channels>
where
    Channels: FeagiRequestResponseChannelBackend,
{
    responder: FeagiResponder<FeagiWorkerRequest<CustomRequest>, FeagiWorkerResponse<CustomResponse>, Channels>,
}

impl<CustomRequest, CustomResponse, Channels> FeagiWorkerCommands<CustomRequest, CustomResponse, Channels>
where
    Channels: FeagiRequestResponseChannelBackend,
{
    /// Wraps the responder half of the pair whose requester is given to `FeagiWorker::new`
    pub fn new(responder: FeagiResponder<FeagiWorkerRequest<CustomRequest>, FeagiWorkerResponse<CustomResponse>, Channels>) -> Self {
        FeagiWorkerCommands { responder }
    }

    /// Handles one command with `handler` if one is queued, otherwise returns `Idle` immediately
    pub async fn try_handle_next<Handler>(&self, handler: &mut Handler) -> FeagiWorkerStep
    where
        Handler: FeagiWorkerCommandHandler<CustomRequest = CustomRequest, CustomResponse = CustomResponse>,
    {
        let mut step = FeagiWorkerStep::Handled;
        let result = self.responder.try_process_next(Self::command_processor(handler, &mut step)).await;
        Self::step_from_result(result, step)
    }

    /// Waits for the next command, then handles it with `handler`
    ///
    /// This future can be raced against other work (as in a `select`), but only against branches
    /// that resolve rapidly. If another branch wins while a command is being handled, the
    /// handling is cut short: the `FeagiWorker` receives `ResponderDroppedRequest`, and a cut
    /// short `on_stop` never finishes its cleanup
    pub async fn async_handle_next<Handler>(&self, handler: &mut Handler) -> FeagiWorkerStep
    where
        Handler: FeagiWorkerCommandHandler<CustomRequest = CustomRequest, CustomResponse = CustomResponse>,
    {
        let mut step = FeagiWorkerStep::Handled;
        let result = self.responder.async_process_next(Self::command_processor(handler, &mut step)).await;
        Self::step_from_result(result, step)
    }

    /// Handles commands until the worker is stopped or its `FeagiWorker` is dropped. For workers
    /// that do nothing but answer commands
    pub async fn run_until_stopped<Handler>(&self, handler: &mut Handler)
    where
        Handler: FeagiWorkerCommandHandler<CustomRequest = CustomRequest, CustomResponse = CustomResponse>,
    {
        loop {
            match self.async_handle_next(handler).await {
                FeagiWorkerStep::Stopped | FeagiWorkerStep::Disconnected => return,
                FeagiWorkerStep::Handled | FeagiWorkerStep::Idle => {}
            }
        }
    }

    /// Dispatches a command to `handler` and wraps its result in the matching reply variant
    fn command_processor<'a, Handler>(
        handler: &'a mut Handler,
        step: &'a mut FeagiWorkerStep,
    ) -> impl AsyncFnOnce(FeagiWorkerRequest<CustomRequest>) -> FeagiWorkerResponse<CustomResponse> + 'a
    where
        Handler: FeagiWorkerCommandHandler<CustomRequest = CustomRequest, CustomResponse = CustomResponse>,
    {
        async move |request| match request {
            FeagiWorkerRequest::Stop => {
                handler.on_stop().await;
                *step = FeagiWorkerStep::Stopped;
                FeagiWorkerResponse::Acknowledged
            }
            FeagiWorkerRequest::Custom(request) => FeagiWorkerResponse::Custom(handler.on_custom(request).await),
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
