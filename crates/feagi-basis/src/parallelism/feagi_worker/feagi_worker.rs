//! A worker is a long running body of code, run in parallel by some backend (a std thread, an
//! embassy task, ...). Its parent holds a `FeagiWorkerHandle`, which sends notifications to the
//! worker over a feagi request / response channel. The worker owns the responding half and must
//! keep answering notifications, replying to `Stop` before it exits.

use core::marker::PhantomData;
use crate::parallelism::feagi_channel::channel_error::FeagiRequestError;
use crate::parallelism::feagi_channel::feagi_channel::{FeagiChannelReceiver, FeagiChannelSender};
use crate::parallelism::feagi_channel::request_response::feagi_request_response::{FeagiRequester, ReplySlot, RequestSlot};
use crate::parallelism::feagi_worker::notifications::{FeagiWorkerNotification, FeagiWorkerReply};
#[cfg(feature = "alloc")]
use crate::parallelism::feagi_channel::feagi_channel::{FeagiChannelReceiverDynamic, FeagiChannelSenderDynamic};
#[cfg(feature = "alloc")]
use crate::parallelism::feagi_channel::request_response::feagi_request_response::{FeagiReplySenderDynamic, FeagiRequestResponseDynamicBuilder, FeagiResponderDynamic};

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
    FeagiChannelSenderDynamic<RequestSlot<FeagiWorkerNotification<CustomNotification>, FeagiReplySenderDynamic<FeagiWorkerReply<CustomReply>>>>,
    FeagiReplySenderDynamic<FeagiWorkerReply<CustomReply>>,
    FeagiChannelReceiverDynamic<ReplySlot<FeagiWorkerReply<CustomReply>>>,
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
        Spawner: FeagiWorkerSpawner<Body, BackendHandle = BackendHandle>,
    {
        let (notifier, notifications) = FeagiRequestResponseDynamicBuilder::new(notification_queue_capacity).create();
        let backend_handle = spawner.spawn_worker(create_body(notifications))?;
        Ok(FeagiWorkerHandle::new(notifier, backend_handle))
    }
}

//endregion
