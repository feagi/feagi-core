//! Request / response over feagi channels

use crate::parallelism::feagi_channel::channel_error::{FeagiChannelReceiveError, FeagiRequestError};
#[cfg(feature = "alloc")]
use crate::parallelism::feagi_channel::feagi_channel::{FeagiChannelDynamicBuilder, FeagiChannelReceiverDynamic, FeagiChannelSenderDynamic};
use crate::parallelism::feagi_channel::feagi_channel::{FeagiChannelReceiver, FeagiChannelSender};
use crate::parallelism::feagi_channel::recycling::Recyclable;
use core::marker::PhantomData;
use core::ops::DerefMut;

/// Marks a type that builds feagi requester / responder pairs
pub trait FeagiRequestResponseBuilder<Request: Recyclable, Response: Recyclable> {
    /// Requesting half produced by this builder
    type Requester;
    /// Responding half produced by this builder
    type Responder;
}

//region Slots

/// Contents of one request channel slot
pub struct RequestSlot<Request, ReplySender> {
    sequence: u32,
    request: Request,
    /// `None` when the slot is recycled, or once the responder has taken it
    reply_sender: Option<ReplySender>,
}

impl<Request: Recyclable, ReplySender> Recyclable for RequestSlot<Request, ReplySender> {
    fn new_for_recycler() -> Self {
        RequestSlot {
            sequence: 0,
            request: Request::new_for_recycler(),
            reply_sender: None,
        }
    }

    fn recycle_self(&mut self) {
        self.request.recycle_self();
        self.reply_sender = None;
    }
}

/// Whether a reply slot holds a real response
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReplyStatus {
    Responded,
    NotResponded,
}

/// Contents of one reply channel slot
pub struct ReplySlot<Response> {
    sequence: u32,
    status: ReplyStatus,
    response: Response,
}

impl<Response: Recyclable> Recyclable for ReplySlot<Response> {
    fn new_for_recycler() -> Self {
        ReplySlot {
            sequence: 0,
            status: ReplyStatus::NotResponded,
            response: Response::new_for_recycler(),
        }
    }

    fn recycle_self(&mut self) {
        self.status = ReplyStatus::NotResponded;
        self.response.recycle_self();
    }
}

//endregion

//region Requester

/// Sends requests to a responder and awaits the matching response
pub struct FeagiRequester<Request, Response, RequestSender, ReplySender, ReplyReceiver> {
    request_sender: RequestSender,
    /// Copied into every request so the responder can reply
    reply_sender: ReplySender,
    reply_receiver: ReplyReceiver,
    next_sequence: u32,
    /// Sequence of a sent request whose reply was never read, because its future was dropped
    unread_reply_sequence: Option<u32>,
    _types: PhantomData<fn(Request) -> Response>,
}

impl<Request, Response, RequestSender, ReplySender, ReplyReceiver> FeagiRequester<Request, Response, RequestSender, ReplySender, ReplyReceiver>
where
    Request: Recyclable,
    Response: Recyclable,
    RequestSender: FeagiChannelSender<RequestSlot<Request, ReplySender>>,
    ReplySender: FeagiChannelSender<ReplySlot<Response>> + Clone,
    ReplyReceiver: FeagiChannelReceiver<ReplySlot<Response>>,
{
    /// Creates a requester from a request channel sender and this requester's own reply channel
    pub(crate) fn new(request_sender: RequestSender, reply_sender: ReplySender, reply_receiver: ReplyReceiver) -> Self {
        FeagiRequester {
            request_sender,
            reply_sender,
            reply_receiver,
            next_sequence: 0,
            unread_reply_sequence: None,
            _types: PhantomData,
        }
    }

    /// Sends `request` and waits for its response
    pub async fn make_request(&mut self, request: Request) -> Result<Response, FeagiRequestError> {
        self.make_request_in_place(
            |slot_request| *slot_request = request,
            |slot_response| core::mem::replace(slot_response, Response::new_for_recycler()),
        )
        .await
    }

    /// Writes a request directly into the channel slot with `write_request`, waits for the
    /// response, then reads it in place with `read_response`
    ///
    /// Avoids moving or constructing either value. `read_response` sees the response before its
    /// slot is recycled
    pub async fn make_request_in_place<Output>(
        &mut self,
        write_request: impl FnOnce(&mut Request),
        read_response: impl FnOnce(&mut Response) -> Output,
    ) -> Result<Output, FeagiRequestError> {
        // A previous request's future was dropped before its reply arrived. Discard that reply
        // first so this requester never has more than one reply in flight
        if let Some(unread_sequence) = self.unread_reply_sequence {
            drop(Self::await_reply(&self.reply_receiver, unread_sequence).await?);
            self.unread_reply_sequence = None;
        }

        let sequence = self.next_sequence;
        self.next_sequence = self.next_sequence.wrapping_add(1);

        // The slot is published to the responder when the guard drops at the end of this block.
        // The reply sender is set last, so a panic in `write_request` publishes a slot the
        // responder skips
        {
            let mut slot = self.request_sender.async_send_ref().await?;
            slot.sequence = sequence;
            write_request(&mut slot.request);
            slot.reply_sender = Some(self.reply_sender.clone());
        }
        self.unread_reply_sequence = Some(sequence);

        let mut reply = Self::await_reply(&self.reply_receiver, sequence).await?;
        self.unread_reply_sequence = None;
        match reply.status {
            ReplyStatus::Responded => Ok(read_response(&mut reply.response)),
            ReplyStatus::NotResponded => Err(FeagiRequestError::ResponderDroppedRequest),
        }
    }

    /// Waits for the reply tagged with `sequence`, discarding any other replies
    async fn await_reply(
        reply_receiver: &ReplyReceiver,
        sequence: u32,
    ) -> Result<<ReplyReceiver as FeagiChannelReceiver<ReplySlot<Response>>>::RecvGuard<'_>, FeagiRequestError> {
        loop {
            let reply = reply_receiver.async_recv_ref().await?;
            if reply.sequence == sequence {
                return Ok(reply);
            }
        }
    }
}

//endregion

//region Responder

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
    ReplySender: FeagiChannelSender<ReplySlot<Response>>,
{
    request_receiver: RequestReceiver,
    _types: PhantomData<fn(Request, ReplySender) -> Response>,
}

impl<Request, Response, RequestReceiver, ReplySender> Drop for FeagiResponder<Request, Response, RequestReceiver, ReplySender>
where
    Request: Recyclable,
    Response: Recyclable,
    RequestReceiver: FeagiChannelReceiver<RequestSlot<Request, ReplySender>>,
    ReplySender: FeagiChannelSender<ReplySlot<Response>>,
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

impl<Request, Response, RequestReceiver, ReplySender> FeagiResponder<Request, Response, RequestReceiver, ReplySender>
where
    Request: Recyclable,
    Response: Recyclable,
    RequestReceiver: FeagiChannelReceiver<RequestSlot<Request, ReplySender>>,
    ReplySender: FeagiChannelSender<ReplySlot<Response>>,
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

//endregion

//region Dynamic

/// Reply sender carried inside dynamic requests
#[cfg(feature = "alloc")]
pub type FeagiReplySenderDynamic<Response> = FeagiChannelSenderDynamic<ReplySlot<Response>>;

/// Requester using heap allocated channels. Cloning creates another requester to the same
/// responder, with its own reply channel
#[cfg(feature = "alloc")]
pub type FeagiRequesterDynamic<Request, Response> = FeagiRequester<
    Request,
    Response,
    FeagiChannelSenderDynamic<RequestSlot<Request, FeagiReplySenderDynamic<Response>>>,
    FeagiReplySenderDynamic<Response>,
    FeagiChannelReceiverDynamic<ReplySlot<Response>>,
>;

/// Responder using heap allocated channels
#[cfg(feature = "alloc")]
pub type FeagiResponderDynamic<Request, Response> = FeagiResponder<
    Request,
    Response,
    FeagiChannelReceiverDynamic<RequestSlot<Request, FeagiReplySenderDynamic<Response>>>,
    FeagiReplySenderDynamic<Response>,
>;

/// Creates requester / responder pairs over heap allocated channels
#[cfg(feature = "alloc")]
pub struct FeagiRequestResponseDynamicBuilder<Request: Recyclable, Response: Recyclable> {
    request_queue_capacity: usize,
    _types: PhantomData<fn(Request) -> Response>,
}

#[cfg(feature = "alloc")]
impl<Request: Recyclable, Response: Recyclable> FeagiRequestResponseDynamicBuilder<Request, Response> {
    /// `request_queue_capacity` is how many requests can be queued for the responder at once
    pub const fn new(request_queue_capacity: usize) -> Self {
        FeagiRequestResponseDynamicBuilder {
            request_queue_capacity,
            _types: PhantomData,
        }
    }

    /// Creates a new, independent requester and responder. Clone the requester for more
    /// requesters to the same responder
    pub fn create(&self) -> (FeagiRequesterDynamic<Request, Response>, FeagiResponderDynamic<Request, Response>) {
        let (request_sender, request_receiver) = FeagiChannelDynamicBuilder::new(self.request_queue_capacity).create_channels();
        (
            FeagiRequesterDynamic::new_with_reply_channel(request_sender),
            FeagiResponderDynamic::new(request_receiver),
        )
    }
}

#[cfg(feature = "alloc")]
impl<Request: Recyclable, Response: Recyclable> FeagiRequestResponseBuilder<Request, Response>
    for FeagiRequestResponseDynamicBuilder<Request, Response>
{
    type Requester = FeagiRequesterDynamic<Request, Response>;
    type Responder = FeagiResponderDynamic<Request, Response>;
}

#[cfg(feature = "alloc")]
impl<Request: Recyclable, Response: Recyclable> FeagiRequesterDynamic<Request, Response> {
    /// Creates a requester with a newly allocated reply channel of capacity 1
    fn new_with_reply_channel(request_sender: FeagiChannelSenderDynamic<RequestSlot<Request, FeagiReplySenderDynamic<Response>>>) -> Self {
        let (reply_sender, reply_receiver) = FeagiChannelDynamicBuilder::new(1).create_channels();
        FeagiRequester::new(request_sender, reply_sender, reply_receiver)
    }
}

#[cfg(feature = "alloc")]
impl<Request: Recyclable, Response: Recyclable> Clone for FeagiRequesterDynamic<Request, Response> {
    fn clone(&self) -> Self {
        FeagiRequesterDynamic::new_with_reply_channel(self.request_sender.clone())
    }
}

//endregion
