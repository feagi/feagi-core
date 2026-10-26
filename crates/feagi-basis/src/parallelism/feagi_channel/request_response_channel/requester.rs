use core::marker::PhantomData;
use crate::parallelism::feagi_channel::base_channels::base_channels::{FeagiChannelReceiver, FeagiChannelTransmitter};
use crate::parallelism::feagi_channel::channel_error::FeagiRequestError;
use crate::parallelism::feagi_channel::recycling::Recyclable;
use crate::parallelism::feagi_channel::request_response_channel::channel_backend::FeagiRequestResponseChannelBackend;
use crate::parallelism::feagi_channel::request_response_channel::shared_structs::{ReplySlot, ReplyStatus};


/// Sends requests to a responder and awaits the matching response
pub struct FeagiRequester<Request, Response, Channels>
where
    Request: Recyclable,
    Response: Recyclable,
    Channels: FeagiRequestResponseChannelBackend,
{
    pub(crate) request_sender: Channels::RequestTransmitter<Request, Response>,
    /// Copied into every request so the responder can reply
    pub(crate) reply_sender: Channels::ReplyTransmitter<Response>,
    pub(crate) reply_receiver: Channels::ReplyReceiver<Response>,
    pub(crate) next_sequence: u32,
    /// Sequence of a sent request whose reply was never read, because its future was dropped
    pub(crate) unread_reply_sequence: Option<u32>,
    pub(crate) _types: PhantomData<fn(Request) -> Response>,
}

impl<Request, Response, Channels> FeagiRequester<Request, Response, Channels>
where
    Request: Recyclable,
    Response: Recyclable,
    Channels: FeagiRequestResponseChannelBackend,
{
    /// Creates a requester from a request channel sender and this requester's own reply channel
    pub(crate) fn new(
        request_sender: Channels::RequestTransmitter<Request, Response>,
        reply_sender: Channels::ReplyTransmitter<Response>,
        reply_receiver: Channels::ReplyReceiver<Response>,
    ) -> Self {
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
        reply_receiver: &Channels::ReplyReceiver<Response>,
        sequence: u32,
    ) -> Result<<Channels::ReplyReceiver<Response> as FeagiChannelReceiver<ReplySlot<Response>>>::RecvGuard<'_>, FeagiRequestError> {
        loop {
            let reply = reply_receiver.async_recv_ref().await?;
            if reply.sequence == sequence {
                return Ok(reply);
            }
        }
    }
}
