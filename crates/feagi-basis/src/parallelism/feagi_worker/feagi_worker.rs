// TODO: `FeagiWorker` refuses requests after an acknowledged `stop` only to avoid a race in
// request / response: a request published after the responder's drop has drained its queue, but
// before its receiver is dropped, is never answered and its requester waits forever. This still
// affects workers that exit on their own without being stopped. Fix properly once thingbuf is
// forked so receivers can close the channel before draining

use crate::parallelism::feagi_channel::channel_error::{FeagiChannelTransmitError, FeagiRequestError};
use crate::parallelism::feagi_channel::request_response_channel::FeagiRequestResponseChannelBackend;
use crate::parallelism::feagi_channel::request_response_channel::FeagiRequester;
use crate::parallelism::feagi_worker::worker_request_response::{FeagiWorkerRequest, FeagiWorkerResponse};

#[cfg(feature = "std")]
use crate::parallelism::feagi_worker::spawner_implementations::std_thread::FeagiStdThreadSpawner;
use crate::parallelism::feagi_worker::worker_spawner_implementation::WorkerSpawnerImplementation;

/// Runs some task separately depending on context. Runs on another thread in std, and on
/// another task in embassy. THe interface is generally uniform irrespective of being in
/// std or no-std modes
pub struct FeagiWorker<Spawner, Channels, CustomRequest, CustomResponse>
where
    Spawner: WorkerSpawnerImplementation,
    Channels: FeagiRequestResponseChannelBackend,
{
    requester: FeagiRequester<FeagiWorkerRequest<CustomRequest>, FeagiWorkerResponse<CustomResponse>, Channels>,
    handle: Spawner::WorkerInternalHandle,
    /// Set once the worker has acknowledged `stop`. Later requests are refused without sending
    stopped: bool,
}

impl<Spawner, Channels, CustomRequest, CustomResponse> FeagiWorker<Spawner, Channels, CustomRequest, CustomResponse>
where
    Spawner: WorkerSpawnerImplementation,
    Channels: FeagiRequestResponseChannelBackend,
{
    #[doc(hidden)] // Probably shouldnt be called directly by other than `FeagiWorkerBuilder`
    /// Meant be called by `FeagiWorkerBuilder::build_worker` implementations, spawns a new
    /// (running) worker
    pub fn new(
        requester: FeagiRequester<FeagiWorkerRequest<CustomRequest>, FeagiWorkerResponse<CustomResponse>, Channels>,
        handle: Spawner::WorkerInternalHandle,
    ) -> Self {
        Self {
            requester,
            handle,
            stopped: false,
        }
    }

    /// Sends a custom request and awaits for the worker's custom reply
    ///
    /// Errors with `RequestNotSent` if the worker has been stopped or has already exited, and
    /// with `UnexpectedResponse` if the worker replied with anything but a custom response
    pub async fn make_request(&mut self, request: CustomRequest) -> Result<CustomResponse, FeagiRequestError> {
        self.refuse_if_stopped()?;
        match self.requester.make_request(FeagiWorkerRequest::Custom(request)).await? {
            FeagiWorkerResponse::Custom(response) => Ok(response),
            _ => Err(FeagiRequestError::UnexpectedResponse),
        }
    }

    /// Tells the worker to stop and waits until it has acknowledged. The worker exits after
    /// replying, so it may still be finishing when this returns
    ///
    /// Errors with `RequestNotSent` if the worker has been stopped or has already exited, and
    /// with `UnexpectedResponse` if the worker replied with anything but an acknowledgement
    pub async fn stop(&mut self) -> Result<(), FeagiRequestError> {
        self.refuse_if_stopped()?;
        match self.requester.make_request(FeagiWorkerRequest::Stop).await? {
            FeagiWorkerResponse::Acknowledged => {
                self.stopped = true;
                Ok(())
            }
            _ => Err(FeagiRequestError::UnexpectedResponse),
        }
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
impl<Channels, CustomRequest, CustomResponse> FeagiWorker<FeagiStdThreadSpawner, Channels, CustomRequest, CustomResponse>
where
    Channels: FeagiRequestResponseChannelBackend,
{
    /// Blocks until the worker's thread exits. Errors with the panic payload if it panicked
    ///
    /// The request channel is dropped first, so a worker that exits once its channel closes
    /// is not left waiting on this handle
    pub fn join(self) -> std::thread::Result<()> {
        drop(self.requester);
        self.handle.join()
    }
}
