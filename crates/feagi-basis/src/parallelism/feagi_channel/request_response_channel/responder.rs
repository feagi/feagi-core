use std::marker::PhantomData;
use std::ops::DerefMut;
use crate::parallelism::feagi_channel::base_channels::basis::{FeagiChannelReceiver, FeagiChannelTransmitter};
use crate::parallelism::feagi_channel::channel_error::FeagiChannelReceiveError;
use crate::parallelism::feagi_channel::recycling::Recyclable;
use crate::parallelism::feagi_channel::request_response_channel::basis::{ReplySlot, ReplyStatus, RequestSlot};

/// Receives requests from any number of requesters and replies to each one
///
/// When dropped, every request still queued is answered as not responded, so no requester is left
/// waiting on a responder that no longer exists
// NOTE: Do NOT allow this to be clonable! There is exactly one responder per request channel
pub struct FeagiResponder<Request, Response, RequestReceiver, ReplySender>
where
    Request: Recyclable,
    Response: Recyclable,
    RequestReceiver: FeagiChannelReceiver<RequestSlot<Request, ReplySender>>,
    ReplySender: FeagiChannelTransmitter<ReplySlot<Response>>,
{
    request_receiver: RequestReceiver,
    _types: PhantomData<fn(Request, ReplySender) -> Response>,
}

impl<Request, Response, RequestReceiver, ReplySender> FeagiResponder<Request, Response, RequestReceiver, ReplySender>
where
    Request: Recyclable,
    Response: Recyclable,
    RequestReceiver: FeagiChannelReceiver<RequestSlot<Request, ReplySender>>,
    ReplySender: FeagiChannelTransmitter<ReplySlot<Response>>,
{
    /// Creates a responder from the request channel receiver
    pub(crate) fn new(request_receiver: RequestReceiver) -> Self {
        FeagiResponder {
            request_receiver,
            _types: PhantomData,
        }
    }

    /// Waits for the next request, then answers it with `processor`, which takes the request
    /// and returns the response by value
    pub async fn async_process_next<Processor>(&self, processor: Processor) -> Result<(), FeagiChannelReceiveError>
    where
        Processor: AsyncFnOnce(Request) -> Response,
    {
        self.async_process_next_in_place(Self::by_value_processor(processor)).await
    }

    /// Answers the next request with `processor` if one is queued, otherwise returns
    /// `ChannelEmpty` immediately. `processor` takes the request and returns the response by value
    pub async fn try_process_next<Processor>(&self, processor: Processor) -> Result<(), FeagiChannelReceiveError>
    where
        Processor: AsyncFnOnce(Request) -> Response,
    {
        self.try_process_next_in_place(Self::by_value_processor(processor)).await
    }

    /// Waits for the next request, then answers it with `processor`, which reads the request and
    /// writes the response directly in their channel slots
    pub async fn async_process_next_in_place<Processor>(&self, processor: Processor) -> Result<(), FeagiChannelReceiveError>
    where
        Processor: AsyncFnOnce(&mut Request, &mut Response),
    {
        let slot = self.request_receiver.async_recv_ref().await?;
        Self::respond_in_place(slot, processor).await;
        Ok(())
    }

    /// Answers the next request with `processor` if one is queued, otherwise returns
    /// `ChannelEmpty` immediately. `processor` reads the request and writes the response directly
    /// in their channel slots
    pub async fn try_process_next_in_place<Processor>(&self, processor: Processor) -> Result<(), FeagiChannelReceiveError>
    where
        Processor: AsyncFnOnce(&mut Request, &mut Response),
    {
        let slot = self.request_receiver.try_recv_ref()?;
        Self::respond_in_place(slot, processor).await;
        Ok(())
    }

    /// Runs `processor` on a received request slot and replies to its requester
    ///
    /// The reply slot is claimed before processing and marked as not responded. If `processor`
    /// is cancelled or panics, dropping the reply guard still publishes the slot, so the requester
    /// receives `ResponderDroppedRequest` instead of waiting forever
    async fn respond_in_place<Processor>(mut slot: impl DerefMut<Target = RequestSlot<Request, ReplySender>>, processor: Processor)
    where
        Processor: AsyncFnOnce(&mut Request, &mut Response),
    {
        let slot = &mut *slot;
        // The requester panicked while writing this request, nobody is waiting for it
        let Some(reply_sender) = slot.reply_sender.take() else {
            return;
        };
        // Closed means the requester is gone, so the request is skipped. Full cannot happen since
        // each requester has at most one reply in flight
        let Ok(mut reply) = reply_sender.try_send_ref() else {
            return;
        };
        reply.sequence = slot.sequence;
        reply.status = ReplyStatus::NotResponded;
        processor(&mut slot.request, &mut reply.response).await;
        reply.status = ReplyStatus::Responded;
    }

    /// Adapts a by-value processor to the in-place form
    fn by_value_processor<Processor>(processor: Processor) -> impl AsyncFnOnce(&mut Request, &mut Response)
    where
        Processor: AsyncFnOnce(Request) -> Response,
    {
        async move |slot_request: &mut Request, slot_response: &mut Response| {
            let request = core::mem::replace(slot_request, Request::new_for_recycler());
            *slot_response = processor(request).await;
        }
    }
}

impl<Request, Response, RequestReceiver, ReplySender> Drop
for FeagiResponder<Request, Response, RequestReceiver, ReplySender>
where
    Request: Recyclable,
    Response: Recyclable,
    RequestReceiver: FeagiChannelReceiver<RequestSlot<Request, ReplySender>>,
    ReplySender: FeagiChannelTransmitter<ReplySlot<Response>>,
{
    // A request published after this drain finishes but before the receiver is dropped is not
    // answered. thingbuf receivers cannot close the channel ahead of their own drop
    fn drop(&mut self) {
        while let Ok(mut slot) = self.request_receiver.try_recv_ref() {
            let Some(reply_sender) = slot.reply_sender.take() else {
                continue;
            };
            if let Ok(mut reply) = reply_sender.try_send_ref() {
                reply.sequence = slot.sequence;
                reply.status = ReplyStatus::NotResponded;
            };
        }
    }
}