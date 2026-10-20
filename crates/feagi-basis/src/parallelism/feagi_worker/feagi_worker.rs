// TODO: `FeagiWorker` refuses requests after an acknowledged `stop` only to avoid a race in
// request / response: a request published after the responder's drop has drained its queue, but
// before its receiver is dropped, is never answered and its requester waits forever. This still
// affects workers that exit on their own without being stopped. Fix properly once thingbuf is
// forked so receivers can close the channel before draining

use crate::parallelism::feagi_channel::channel_error::{FeagiChannelSendError, FeagiRequestError};
use crate::parallelism::feagi_channel::request_response::feagi_request_response::FeagiRequester;
use crate::parallelism::feagi_worker::feagi_worker_builder::WorkerChannelImplementation;
use crate::parallelism::feagi_worker::worker_requests::{FeagiWorkerRequest, FeagiWorkerResponse};
#[cfg(feature = "std")]
use crate::parallelism::feagi_worker::worker_spawner_implementation::FeagiStdThreadSpawner;
use crate::parallelism::feagi_worker::worker_spawner_implementation::WorkerSpawnerImplementation;

/// Runs some task separately depending on context. Runs on another thread in std, and on
/// another task in embassy. THe interface is generally uniform irrespective of being in
/// std or no-std modes
pub struct FeagiWorker<SpawnerImplementation, ChannelImplementation, CustomCommandRequest, CustomCommandResponse>
where
    SpawnerImplementation: WorkerSpawnerImplementation,
    ChannelImplementation: WorkerChannelImplementation,
{
    request_maker: FeagiRequester<
        FeagiWorkerRequest<CustomCommandRequest>,
        FeagiWorkerResponse<CustomCommandResponse>,
        ChannelImplementation::RequestSender<FeagiWorkerRequest<CustomCommandRequest>, FeagiWorkerResponse<CustomCommandResponse>>,
        ChannelImplementation::ReplySender<FeagiWorkerResponse<CustomCommandResponse>>,
        ChannelImplementation::ReplyReceiver<FeagiWorkerResponse<CustomCommandResponse>>,
    >,
    handle: SpawnerImplementation::WorkerInternalHandle,
    /// Set once the worker has acknowledged `stop`. Later requests are refused without sending
    stopped: bool,
}

impl<SpawnerImplementation, ChannelImplementation, CustomCommandRequest, CustomCommandResponse>
    FeagiWorker<SpawnerImplementation, ChannelImplementation, CustomCommandRequest, CustomCommandResponse>
where
    SpawnerImplementation: WorkerSpawnerImplementation,
    ChannelImplementation: WorkerChannelImplementation,
{
    /// Meant be called by `FeagiWorkerBuilder::build_worker` implementations, spawns a new
    /// (running) worker
    pub fn new(
        request_maker: FeagiRequester<
            FeagiWorkerRequest<CustomCommandRequest>,
            FeagiWorkerResponse<CustomCommandResponse>,
            ChannelImplementation::RequestSender<FeagiWorkerRequest<CustomCommandRequest>, FeagiWorkerResponse<CustomCommandResponse>>,
            ChannelImplementation::ReplySender<FeagiWorkerResponse<CustomCommandResponse>>,
            ChannelImplementation::ReplyReceiver<FeagiWorkerResponse<CustomCommandResponse>>,
        >,
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
            return Err(FeagiRequestError::RequestNotSent(FeagiChannelSendError::ChannelClosed));
        }
        Ok(())
    }
}

#[cfg(feature = "std")]
impl<ChannelImplementation, CustomCommandRequest, CustomCommandResponse>
    FeagiWorker<FeagiStdThreadSpawner, ChannelImplementation, CustomCommandRequest, CustomCommandResponse>
where
    ChannelImplementation: WorkerChannelImplementation,
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

/*
//! A worker is a long running body of code, run in parallel by some backend (a std thread, an
//! embassy task, ...). Its parent holds a `FeagiWorkerHandle`, which sends notifications to the
//! worker over a feagi request / response channel. The worker owns the responding half and must
//! keep answering notifications, replying to `Stop` before it exits.

use crate::parallelism::feagi_channel::channel_error::FeagiRequestError;
use crate::parallelism::feagi_channel::feagi_channel::{FeagiChannelReceiver, FeagiChannelSender};
#[cfg(feature = "alloc")]
use crate::parallelism::feagi_channel::request_response::feagi_request_response::{FeagiReplyReceiverDynamic, FeagiReplySenderDynamic, FeagiRequestResponseDynamicBuilder, FeagiRequestSenderDynamic, FeagiResponderDynamic};
use crate::parallelism::feagi_channel::request_response::feagi_request_response::{FeagiRequester, ReplySlot, RequestSlot};
use crate::parallelism::feagi_worker::notifications::{FeagiWorkerNotification, FeagiWorkerReply};
use core::marker::PhantomData;

//region Spawner

/// A backend that runs a worker body in parallel
///
/// `Body` is whatever the backend can run: a closure for std threads, a spawn token for embassy.
/// Spawners are expected to be cheap to copy so worker bodies can take one and spawn children
pub trait FeagiWorkerSpawner<Body> {
    /// What the backend returns for a running worker, such as a thread join handle
    type BackendHandle;
    type SpawnError: core::error::Error;

    /// Starts running `body` in parallel
    fn spawn_worker(&self, body: Body) -> Result<Self::BackendHandle, Self::SpawnError>;
}

//endregion

//region Handle

/// Parent side of a running worker. Sends notifications and waits for the worker's replies
pub struct FeagiWorkerHandle<CustomNotification, CustomReply, RequestSender, ReplySender, ReplyReceiver, BackendHandle> {
    notifier: FeagiRequester<FeagiWorkerNotification<CustomNotification>, FeagiWorkerReply<CustomReply>, RequestSender, ReplySender, ReplyReceiver>,
    backend_handle: BackendHandle,
    _types: PhantomData<fn(CustomNotification) -> CustomReply>,
}

impl<CustomNotification, CustomReply, RequestSender, ReplySender, ReplyReceiver, BackendHandle>
FeagiWorkerHandle<CustomNotification, CustomReply, RequestSender, ReplySender, ReplyReceiver, BackendHandle>
where
    RequestSender: FeagiChannelSender<RequestSlot<FeagiWorkerNotification<CustomNotification>, ReplySender>>,
    ReplySender: FeagiChannelSender<ReplySlot<FeagiWorkerReply<CustomReply>>> + Clone,
    ReplyReceiver: FeagiChannelReceiver<ReplySlot<FeagiWorkerReply<CustomReply>>>,
{
    /// Creates a handle from the requesting half of the worker's notification channel and the
    /// backend's handle to the running worker
    pub fn new(
        notifier: FeagiRequester<FeagiWorkerNotification<CustomNotification>, FeagiWorkerReply<CustomReply>, RequestSender, ReplySender, ReplyReceiver>,
        backend_handle: BackendHandle,
    ) -> Self {
        FeagiWorkerHandle {
            notifier,
            backend_handle,
            _types: PhantomData,
        }
    }

    /// Sends a custom notification and waits for the worker's reply
    pub async fn notify(&mut self, notification: CustomNotification) -> Result<FeagiWorkerReply<CustomReply>, FeagiRequestError> {
        self.notifier.make_request(FeagiWorkerNotification::Custom(notification)).await
    }

    /// Tells the worker to stop and waits until it has acknowledged. The worker exits after
    /// replying, so it may still be finishing when this returns
    ///
    /// Errors with `RequestNotSent` if the worker has already exited
    pub async fn stop(&mut self) -> Result<(), FeagiRequestError> {
        self.notifier.make_request(FeagiWorkerNotification::Stop).await?;
        Ok(())
    }

    /// Gives up the handle, returning the backend's handle to the worker
    pub fn into_backend_handle(self) -> BackendHandle {
        self.backend_handle
    }
}

#[cfg(feature = "std")]
impl<CustomNotification, CustomReply, RequestSender, ReplySender, ReplyReceiver>
FeagiWorkerHandle<CustomNotification, CustomReply, RequestSender, ReplySender, ReplyReceiver, std::thread::JoinHandle<()>>
{
    /// Blocks until the worker's thread exits. Errors with the panic payload if the worker panicked
    pub fn join(self) -> std::thread::Result<()> {
        self.backend_handle.join()
    }
}

//endregion

//region Dynamic

/// Worker side of a dynamic notification channel. Answer notifications with its
/// `async_process_next` / `try_process_next` methods
#[cfg(feature = "alloc")]
pub type FeagiWorkerNotificationsDynamic<CustomNotification, CustomReply> =
FeagiResponderDynamic<FeagiWorkerNotification<CustomNotification>, FeagiWorkerReply<CustomReply>>;

/// Worker handle using heap allocated notification channels
#[cfg(feature = "alloc")]
pub type FeagiWorkerHandleDynamic<CustomNotification, CustomReply, BackendHandle> = FeagiWorkerHandle<
    CustomNotification,
    CustomReply,
    FeagiRequestSenderDynamic<FeagiWorkerNotification<CustomNotification>, FeagiWorkerReply<CustomReply>>,
    FeagiReplySenderDynamic<FeagiWorkerReply<CustomReply>>,
    FeagiReplyReceiverDynamic<FeagiWorkerReply<CustomReply>>,
    BackendHandle,
>;

#[cfg(feature = "alloc")]
impl<CustomNotification, CustomReply, BackendHandle> FeagiWorkerHandleDynamic<CustomNotification, CustomReply, BackendHandle> {
    /// Creates the worker's notification channel, then spawns the body built by `create_body`
    /// on `spawner`. `create_body` receives the worker side of the notification channel
    ///
    /// `notification_queue_capacity` is how many notifications can be queued for the worker at once
    pub fn start<Spawner, Body>(
        spawner: &Spawner,
        notification_queue_capacity: usize,
        create_body: impl FnOnce(FeagiWorkerNotificationsDynamic<CustomNotification, CustomReply>) -> Body,
    ) -> Result<Self, Spawner::SpawnError>
    where
        Spawner: FeagiWorkerSpawner<Body, BackendHandle=BackendHandle>,
    {
        let (notifier, notifications) = FeagiRequestResponseDynamicBuilder::new(notification_queue_capacity).create();
        let backend_handle = spawner.spawn_worker(create_body(notifications))?;
        Ok(FeagiWorkerHandle::new(notifier, backend_handle))
    }
}

//endregion


 */
