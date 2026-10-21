use crate::parallelism::feagi_channel::base_channels::base_traits::{FeagiChannelReceiver, FeagiChannelTransmitter};
use crate::parallelism::feagi_channel::recycling::Recyclable;
use crate::parallelism::feagi_channel::request_response_channel::{FeagiRequester, FeagiResponder};
use crate::parallelism::feagi_channel::request_response_channel::shared_structs::{ReplySlot, RequestSlot};

/// Which channel types (heap allocated, static) a requester / responder pair is built from. All
/// four are always of the same kind, so this is implemented by a zero sized marker per kind
// TODO: Static is disabled until thingbuf is forked / patched. thingbuf 0.1.6 only implements
// `Clone` for `StaticSender` with its default recycle policy, so a `StaticSender` using
// `RecyclableRecyclePolicy` cannot satisfy `ReplyTransmitter: Clone`. The static channel types
// also require `'static` items, so the GAT bounds will need `Request: 'static` /
// `Response: 'static` added when static channels are enabled
pub trait FeagiRequestResponseChannelBackend: Sized {
    /// Sends requests, each carrying a copy of the requester's reply transmitter
    type RequestTransmitter<Request: Recyclable, Response: Recyclable>: FeagiChannelTransmitter<RequestSlot<Request, Self::ReplyTransmitter<Response>>>;
    /// Receives requests on the responder's side
    type RequestReceiver<Request: Recyclable, Response: Recyclable>: FeagiChannelReceiver<RequestSlot<Request, Self::ReplyTransmitter<Response>>>;
    /// Sends replies back to the requester. Cloned into every request
    type ReplyTransmitter<Response: Recyclable>: FeagiChannelTransmitter<ReplySlot<Response>> + Clone;
    /// Receives replies on the requester's side
    type ReplyReceiver<Response: Recyclable>: FeagiChannelReceiver<ReplySlot<Response>>;

    /// Create a channel pairing regardless of backend with default settings. Note that implementations
    /// will have their own more specific constructors as well, but this is meant to be generic!
    fn create_default_channel_pair<
        Request: Recyclable, Response: Recyclable, const QUEUE_SIZE: usize
    >() -> (
        FeagiRequester<Request, Response, Self>,
        FeagiResponder<Request, Response, Self>
    );
}