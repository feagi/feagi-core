use std::marker::PhantomData;
use crate::parallelism::feagi_channel::base_channels::implementations::alloc_channels::FeagiChannelAllocBuilder;
use crate::parallelism::feagi_channel::recycling::{Recyclable, RecyclableRecyclePolicy};
use crate::parallelism::feagi_channel::request_response_channel::basis::{FeagiRequestResponseBuilder, ReplySlot, RequestSlot};
use crate::parallelism::feagi_channel::request_response_channel::requester::FeagiRequester;
use crate::parallelism::feagi_channel::request_response_channel::responder::FeagiResponder;

//region Builder

/// Creates requester / responder pairs over heap allocated channels
pub struct FeagiRequestResponseAllocBuilder<Request: Recyclable, Response: Recyclable> {
    request_queue_capacity: usize,
    _types: PhantomData<fn(Request) -> Response>,
}

impl<Request: Recyclable, Response: Recyclable> FeagiRequestResponseAllocBuilder<Request, Response> {
    /// `request_queue_capacity` is how many requests can be queued for the responder at once
    pub const fn new(request_queue_capacity: usize) -> Self {
        FeagiRequestResponseAllocBuilder {
            request_queue_capacity,
            _types: PhantomData,
        }
    }

    /// Creates a new, independent requester and responder. Clone the requester for more
    /// requesters to the same responder
    pub fn create(&self) -> (FeagiRequesterAlloc<Request, Response>, FeagiResponderAlloc<Request, Response>) {
        let (request_sender, request_receiver) = FeagiChannelAllocBuilder::new(self.request_queue_capacity).create_channels();
        (
            FeagiRequesterAlloc::new_with_reply_channel(request_sender),
            FeagiResponderAlloc::new(request_receiver),
        )
    }
}

impl<Request: Recyclable, Response: Recyclable> FeagiRequestResponseBuilder<Request, Response>
for FeagiRequestResponseAllocBuilder<Request, Response>
{
    type Requester = FeagiRequesterAlloc<Request, Response>;
    type Responder = FeagiResponderAlloc<Request, Response>;
}

//endregion

/// Request transmitter used by alloc requesters
pub type FeagiRequestTransmitterAlloc<Request, Response> =
    thingbuf::mpsc::Sender<RequestSlot<Request, FeagiReplyTransmitterAlloc<Response>>, RecyclableRecyclePolicy<RequestSlot<Request, FeagiReplyTransmitterAlloc<Response>>>>;

/// Request receiver used by alloc responders
pub type FeagiRequestReceiverAlloc<Request, Response> =
    thingbuf::mpsc::Receiver<RequestSlot<Request, FeagiReplyTransmitterAlloc<Response>>, RecyclableRecyclePolicy<RequestSlot<Request, FeagiReplyTransmitterAlloc<Response>>>>;

/// Reply transmitter carried inside alloc requests
pub type FeagiReplyTransmitterAlloc<Response> = 
    thingbuf::mpsc::Sender<ReplySlot<Response>, RecyclableRecyclePolicy<ReplySlot<Response>>>;

/// Reply receiver used by alloc requesters
pub type FeagiReplyReceiverAlloc<Response> = 
    thingbuf::mpsc::Receiver<ReplySlot<Response>, RecyclableRecyclePolicy<ReplySlot<Response>>>;

/// Requester using heap allocated channels. Cloning creates another requester to the same
/// responder, with its own reply channel
pub type FeagiRequesterAlloc<Request, Response> = FeagiRequester<
    Request,
    Response,
    FeagiRequestTransmitterAlloc<Request, Response>,
    FeagiReplyTransmitterAlloc<Response>,
    FeagiReplyReceiverAlloc<Response>,
>;

/// Responder using heap allocated channels
pub type FeagiResponderAlloc<Request, Response> = FeagiResponder<
    Request,
    Response,
    FeagiRequestReceiverAlloc<Request, Response>,
    FeagiReplyTransmitterAlloc<Response>,
>;


impl<Request: Recyclable, Response: Recyclable> FeagiRequesterAlloc<Request, Response> {
    /// Creates a requester with a newly allocated reply channel of capacity 1
    fn new_with_reply_channel(request_sender: FeagiRequestTransmitterAlloc<Request, Response>) -> Self {
        let (reply_sender, reply_receiver) = FeagiChannelAllocBuilder::new(1).create_channels();
        FeagiRequester::new(request_sender, reply_sender, reply_receiver)
    }
}


impl<Request: Recyclable, Response: Recyclable> Clone for FeagiRequesterAlloc<Request, Response> {
    fn clone(&self) -> Self {
        FeagiRequesterAlloc::new_with_reply_channel(self.request_sender.clone())
    }
}