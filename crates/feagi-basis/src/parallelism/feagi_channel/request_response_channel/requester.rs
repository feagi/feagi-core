use core::marker::PhantomData;
use crate::parallelism::feagi_channel::base_channels::base_channels::{FeagiChannelReceiver, FeagiChannelTransmitter};
use crate::parallelism::feagi_channel::recycling::Recyclable;
use crate::parallelism::feagi_channel::request_response_channel::channel_backend::FeagiRequestResponseChannelBackend;
use crate::parallelism::feagi_channel::request_response_channel::feagi_request_response_error::FeagiRequestResponseError;
use crate::parallelism::feagi_channel::request_response_channel::shared_structs::{ReplySlot, ReplyStatus};


/// Sends requests to a responder and awaits the matching response
pub struct FeagiRequester<Request, Response, Backend>
where
    Request: Recyclable,
    Response: Recyclable,
    Backend: FeagiRequestResponseChannelBackend,
{
    pub(crate) request_sender: Backend::RequestTransmitter<Request, Response>,
    /// Copied into every request so the responder can reply
    pub(crate) reply_sender: Backend::ReplyTransmitter<Response>,
    pub(crate) reply_receiver: Backend::ReplyReceiver<Response>,
    pub(crate) next_sequence: u32,
    /// Sequence of a sent request whose reply was never read, because its future was dropped
    pub(crate) unread_reply_sequence: Option<u32>,
    pub(crate) _types: PhantomData<fn(Request) -> Response>,
}

impl<Request, Response, Backend> FeagiRequester<Request, Response, Backend>
where
    Request: Recyclable,
    Response: Recyclable,
    Backend: FeagiRequestResponseChannelBackend,
{
    /// Creates a requester from a request channel sender and this requester's own reply channel
    pub(crate) fn new(
        request_tx: Backend::RequestTransmitter<Request, Response>,
        reply_tx: Backend::ReplyTransmitter<Response>,
        reply_rx: Backend::ReplyReceiver<Response>,
    ) -> Self {
        FeagiRequester {
            request_sender: request_tx,
            reply_sender: reply_tx,
            reply_receiver: reply_rx,
            next_sequence: 0,
            unread_reply_sequence: None,
            _types: PhantomData,
        }
    }

    /// Sends `request` and waits for its response
    pub async fn make_request(&mut self, request: Request) -> Result<Response, FeagiRequestResponseError> {
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
    ) -> Result<Output, FeagiRequestResponseError> {
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
            ReplyStatus::NotResponded => Err(FeagiRequestResponseError::ResponderDroppedRequest),
        }
    }

    /// Waits for the reply tagged with `sequence`, discarding any other replies
    async fn await_reply(
        reply_receiver: &Backend::ReplyReceiver<Response>,
        sequence: u32,
    ) -> Result<<Backend::ReplyReceiver<Response> as FeagiChannelReceiver<ReplySlot<Response>>>::RecvGuard<'_>, FeagiRequestResponseError> {
        loop {
            let reply = reply_receiver.async_recv_ref().await?;
            if reply.sequence == sequence {
                return Ok(reply);
            }
        }
    }
}

#[cfg(feature = "alloc")]
use crate::parallelism::feagi_channel::base_channels::implementations::alloc_channels::FeagiChannelAllocBuilder;
#[cfg(feature = "alloc")]
use crate::parallelism::feagi_channel::request_response_channel::dynamic::FeagiRequestResponseChannelsAllocBackend;

#[cfg(feature = "alloc")]
impl<Request: Recyclable, Response: Recyclable> Clone for FeagiRequester<Request, Response, FeagiRequestResponseChannelsAllocBackend>{
    fn clone(&self) -> Self {

        let reply_channel_capacity = self.reply_sender.capacity();
        let (reply_tx, reply_rx)
            = FeagiChannelAllocBuilder::create_channels(reply_channel_capacity);
        Self::new(
            self.request_sender.clone(),
            reply_tx,
            reply_rx
        )
    }
}

// TODO clone for no-std