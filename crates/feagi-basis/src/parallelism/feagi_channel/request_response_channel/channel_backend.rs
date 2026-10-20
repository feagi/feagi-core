use crate::parallelism::feagi_channel::base_channels::base_traits::{FeagiChannelReceiver, FeagiChannelTransmitter};
use crate::parallelism::feagi_channel::recycling::Recyclable;
use crate::parallelism::feagi_channel::request_response_channel::basis::{ReplySlot, RequestSlot};

/// Which channel types (heap allocated, static) a requester / responder pair is built from. All
/// four are always of the same kind, so this is implemented by a zero sized marker per kind
// TODO: Static is disabled until thingbuf is forked / patched. thingbuf 0.1.6 only implements
// `Clone` for `StaticSender` with its default recycle policy, so a `StaticSender` using
// `RecyclableRecyclePolicy` cannot satisfy `ReplyTransmitter: Clone`. The static channel types
// also require `'static` items, so the GAT bounds will need `Request: 'static` /
// `Response: 'static` added when static channels are enabled
pub trait FeagiRequestResponseChannelBackend {
    /// Sends requests, each carrying a copy of the requester's reply transmitter
    type RequestTransmitter<Request: Recyclable, Response: Recyclable>: FeagiChannelTransmitter<RequestSlot<Request, Self::ReplyTransmitter<Response>>>;
    /// Receives requests on the responder's side
    type RequestReceiver<Request: Recyclable, Response: Recyclable>: FeagiChannelReceiver<RequestSlot<Request, Self::ReplyTransmitter<Response>>>;
    /// Sends replies back to the requester. Cloned into every request
    type ReplyTransmitter<Response: Recyclable>: FeagiChannelTransmitter<ReplySlot<Response>> + Clone;
    /// Receives replies on the requester's side
    type ReplyReceiver<Response: Recyclable>: FeagiChannelReceiver<ReplySlot<Response>>;
}