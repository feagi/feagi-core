use crate::parallelism::feagi_channel::base_channels::implementations::alloc_channels::FeagiChannelAllocBuilder;
use crate::parallelism::feagi_channel::recycling::{Recyclable, RecyclableRecyclePolicy};
use crate::parallelism::feagi_channel::request_response_channel::channel_backend::FeagiRequestResponseChannelBackend;
use crate::parallelism::feagi_channel::request_response_channel::requester::{FeagiRequester};
use crate::parallelism::feagi_channel::request_response_channel::responder::FeagiResponder;
use crate::parallelism::feagi_channel::request_response_channel::shared_structs::{ReplySlot, RequestSlot};

//region Channel Backend

/// Requester / responder pairs use heap allocated channels
#[derive(Debug, Clone, Copy, Default)]
pub struct FeagiRequestResponseChannelsAllocBackend;

impl FeagiRequestResponseChannelBackend for FeagiRequestResponseChannelsAllocBackend {
    type RequestTransmitter<Request: Recyclable, Response: Recyclable> = FeagiRequestTransmitterAlloc<Request, Response>;
    type RequestReceiver<Request: Recyclable, Response: Recyclable> = FeagiRequestReceiverAlloc<Request, Response>;
    type ReplyTransmitter<Response: Recyclable> = FeagiReplyTransmitterAlloc<Response>;
    type ReplyReceiver<Response: Recyclable> = FeagiReplyReceiverAlloc<Response>;

    fn create_default_channel_pair<Request: Recyclable, Response: Recyclable, const QUEUE_SIZE: usize>(
    ) -> (FeagiRequester<Request, Response, Self>, FeagiResponder<Request, Response, Self>) {
        Self::create_channels(QUEUE_SIZE)
    }
}

impl FeagiRequestResponseChannelsAllocBackend {
    /// Creates a new, independent requester and responder. Clone the requester for more
    /// requesters to the same responder. `request_queue_capacity` is how many requests can be
    /// queued for the responder at once. The responser itself gets a capacity of one
    pub fn create_channels<Request: Recyclable, Response: Recyclable>(
        request_queue_capacity: usize,
    ) -> (FeagiRequesterAlloc<Request, Response>, FeagiResponderAlloc<Request, Response>) {
        let (request_tx, request_rx) = FeagiChannelAllocBuilder::create_channels(request_queue_capacity);
        let (reply_tx, reply_rx) = FeagiChannelAllocBuilder::create_channels(1);
        (
            FeagiRequesterAlloc::new(request_tx, reply_tx, reply_rx), 
            FeagiResponderAlloc::new(request_rx),
        )
    }
}

//endregion

/// Requester using heap allocated channels
pub type FeagiRequesterAlloc<Request, Response> = FeagiRequester<Request, Response, FeagiRequestResponseChannelsAllocBackend>;

/// Responder using heap allocated channels
pub type FeagiResponderAlloc<Request, Response> = FeagiResponder<Request, Response, FeagiRequestResponseChannelsAllocBackend>;


//region Request

/// Request transmitter used by alloc requesters
pub(crate) type FeagiRequestTransmitterAlloc<Request, Response> = thingbuf::mpsc::Sender<
    RequestSlot<Request, FeagiReplyTransmitterAlloc<Response>>,
    RecyclableRecyclePolicy<RequestSlot<Request, FeagiReplyTransmitterAlloc<Response>>>,
>;

/// Request receiver used by alloc responders
pub(crate) type FeagiRequestReceiverAlloc<Request, Response> = thingbuf::mpsc::Receiver<
    RequestSlot<Request, FeagiReplyTransmitterAlloc<Response>>,
    RecyclableRecyclePolicy<RequestSlot<Request, FeagiReplyTransmitterAlloc<Response>>>,
>;

//endregion

//region Reply

/// Reply transmitter carried inside alloc requests
pub(crate) type FeagiReplyTransmitterAlloc<Response> = thingbuf::mpsc::Sender<ReplySlot<Response>, RecyclableRecyclePolicy<ReplySlot<Response>>>;

/// Reply receiver used by alloc requesters
pub(crate) type FeagiReplyReceiverAlloc<Response> = thingbuf::mpsc::Receiver<ReplySlot<Response>, RecyclableRecyclePolicy<ReplySlot<Response>>>;

//endregion
