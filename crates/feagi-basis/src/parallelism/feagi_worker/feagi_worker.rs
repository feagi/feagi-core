// TODO: `FeagiWorker` refuses requests after an acknowledged `stop` only to avoid a race in
// request / response: a request published after the responder's drop has drained its queue, but
// before its receiver is dropped, is never answered and its requester waits forever. This still
// affects workers that exit on their own without being stopped. Fix properly once thingbuf is
// forked so receivers can close the channel before draining

use crate::parallelism::feagi_channel::channel_error::{FeagiChannelTransmitError, FeagiRequestError};
use crate::parallelism::feagi_channel::request_response_channel::FeagiRequester;
use crate::parallelism::feagi_channel::request_response_channel::basis::FeagiRequestResponseChannels;
use crate::parallelism::feagi_worker::worker_request_response::{FeagiWorkerRequest, FeagiWorkerResponse};

#[cfg(feature = "std")]
use crate::parallelism::feagi_worker::spawner_implementations::std_thread::FeagiStdThreadSpawner;
use crate::parallelism::feagi_worker::worker_spawner_implementation::WorkerSpawnerImplementation;

/// Runs some task separately depending on context. Runs on another thread in std, and on
/// another task in embassy. THe interface is generally uniform irrespective of being in
/// std or no-std modes
pub struct FeagiWorker<SpawnerImplementation, ChannelImplementation, CustomCommandRequest, CustomCommandResponse>
where
    SpawnerImplementation: WorkerSpawnerImplementation,
    ChannelImplementation: FeagiRequestResponseChannels,
{
    request_maker: FeagiRequester<FeagiWorkerRequest<CustomCommandRequest>, FeagiWorkerResponse<CustomCommandResponse>, ChannelImplementation>,
    handle: SpawnerImplementation::WorkerInternalHandle,
    /// Set once the worker has acknowledged `stop`. Later requests are refused without sending
    stopped: bool,
}

impl<SpawnerImplementation, ChannelImplementation, CustomCommandRequest, CustomCommandResponse>
    FeagiWorker<SpawnerImplementation, ChannelImplementation, CustomCommandRequest, CustomCommandResponse>
where
    SpawnerImplementation: WorkerSpawnerImplementation,
    ChannelImplementation: FeagiRequestResponseChannels,
{
    /// Meant be called by `FeagiWorkerBuilder::build_worker` implementations, spawns a new
    /// (running) worker
    pub fn new(
        request_maker: FeagiRequester<FeagiWorkerRequest<CustomCommandRequest>, FeagiWorkerResponse<CustomCommandResponse>, ChannelImplementation>,
        handle: SpawnerImplementation::WorkerInternalHandle,
    ) -> Self {
        Self {
            request_maker,
            handle,
            stopped: false,
        }
    }

    /// Sends a custom request and awaits for the worker's reply
    ///
    /// Errors with `RequestNotSent` if the worker has been stopped or has already exited
    pub async fn make_request(&mut self, request: CustomCommandRequest) -> Result<FeagiWorkerResponse<CustomCommandResponse>, FeagiRequestError> {
        self.refuse_if_stopped()?;
        self.request_maker.make_request(FeagiWorkerRequest::Custom(request)).await
    }

    /// Tells the worker to stop and waits until it has acknowledged. The worker exits after
    /// replying, so it may still be finishing when this returns
    ///
    /// Errors with `RequestNotSent` if the worker has been stopped or has already exited
    pub async fn stop(&mut self) -> Result<(), FeagiRequestError> {
        self.refuse_if_stopped()?;
        self.request_maker.make_request(FeagiWorkerRequest::Stop).await?;
        self.stopped = true;
        Ok(())
    }

    /// Errors the same way as a request to an exited worker, if `stop` was acknowledged
    fn refuse_if_stopped(&self) -> Result<(), FeagiRequestError> {
        if self.stopped {
            return Err(FeagiRequestError::RequestNotSent(FeagiChannelTransmitError::ChannelClosed));
        }
        Ok(())
    }
}

// Joining is specific to std. Leaving here to ease of understanding as its not really an "implementation"
#[cfg(feature = "std")]
impl<ChannelImplementation, CustomCommandRequest, CustomCommandResponse>
    FeagiWorker<FeagiStdThreadSpawner, ChannelImplementation, CustomCommandRequest, CustomCommandResponse>
where
    ChannelImplementation: FeagiRequestResponseChannels,
{
    /// Blocks until the worker's thread exits. Errors with the panic payload if it panicked
    ///
    /// The request channel is dropped first, so a worker that exits once its channel closes
    /// is not left waiting on this handle
    pub fn join(self) -> std::thread::Result<()> {
        drop(self.request_maker);
        self.handle.join()
    }
}
