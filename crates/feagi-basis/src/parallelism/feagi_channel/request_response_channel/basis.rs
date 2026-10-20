use crate::parallelism::feagi_channel::base_channels::base_traits::{FeagiChannelReceiver, FeagiChannelTransmitter};
use crate::parallelism::feagi_channel::recycling::Recyclable;
use crate::parallelism::feagi_channel::request_response_channel::requester::FeagiRequester;
use crate::parallelism::feagi_channel::request_response_channel::responder::FeagiResponder;

/// Which channel types (heap allocated, static) a requester / responder pair is built from. All
/// four are always of the same kind, so this is implemented by a zero sized marker per kind
// TODO: Static is disabled until thingbuf is forked / patched. thingbuf 0.1.6 only implements
// `Clone` for `StaticSender` with its default recycle policy, so a `StaticSender` using
// `RecyclableRecyclePolicy` cannot satisfy `ReplyTransmitter: Clone`. The static channel types
// also require `'static` items, so the GAT bounds will need `Request: 'static` /
// `Response: 'static` added when static channels are enabled
pub trait FeagiRequestResponseChannels {
    /// Sends requests, each carrying a copy of the requester's reply transmitter
    type RequestTransmitter<Request: Recyclable, Response: Recyclable>: FeagiChannelTransmitter<RequestSlot<Request, Self::ReplyTransmitter<Response>>>;
    /// Receives requests on the responder's side
    type RequestReceiver<Request: Recyclable, Response: Recyclable>: FeagiChannelReceiver<RequestSlot<Request, Self::ReplyTransmitter<Response>>>;
    /// Sends replies back to the requester. Cloned into every request
    type ReplyTransmitter<Response: Recyclable>: FeagiChannelTransmitter<ReplySlot<Response>> + Clone;
    /// Receives replies on the requester's side
    type ReplyReceiver<Response: Recyclable>: FeagiChannelReceiver<ReplySlot<Response>>;
}

/// Marks a type that builds feagi requester / responder pairs
pub trait FeagiRequestResponseBuilder<Request: Recyclable, Response: Recyclable> {
    /// Which channel types the built pairs use
    type Channels: FeagiRequestResponseChannels;
}

/// Requesting half produced by builder `Builder`
pub type FeagiRequestResponseBuilderRequester<Builder, Request, Response> =
    FeagiRequester<Request, Response, <Builder as FeagiRequestResponseBuilder<Request, Response>>::Channels>;

/// Responding half produced by builder `Builder`
pub type FeagiRequestResponseBuilderResponder<Builder, Request, Response> =
    FeagiResponder<Request, Response, <Builder as FeagiRequestResponseBuilder<Request, Response>>::Channels>;

//region Shared Structs

/// Contents of one request channel slot
pub struct RequestSlot<Request, ReplySender> {
    pub(crate) sequence: u32,
    pub(crate) request: Request,
    /// `None` when the slot is recycled, or once the responder has taken it
    pub(crate) reply_sender: Option<ReplySender>,
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
pub enum ReplyStatus {
    Responded,
    NotResponded,
}

/// Contents of one reply channel slot
pub struct ReplySlot<Response> {
    pub(crate) sequence: u32,
    pub(crate) status: ReplyStatus,
    pub(crate) response: Response,
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