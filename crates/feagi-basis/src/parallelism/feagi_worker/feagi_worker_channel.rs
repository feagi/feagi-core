use crate::parallelism::feagi_channel::base_channels::base_traits::{FeagiChannelReceiver, FeagiChannelTransmitter};
use crate::parallelism::feagi_channel::recycling::Recyclable;
use crate::parallelism::feagi_channel::request_response_channel::basis::{ReplySlot, RequestSlot};
use crate::parallelism::feagi_channel::request_response_channel::implementations::dynamic::{FeagiReplyReceiverAlloc, FeagiReplyTransmitterAlloc, FeagiRequestTransmitterAlloc};

/// Which channel types a worker uses to receive requests and send replies (heap allocated,
/// static)
pub trait FeagiWorkerChannels {
    /// Sends requests, each carrying a copy of the requester's reply sender
    type RequestSender<Request: Recyclable, Response: Recyclable>: FeagiChannelTransmitter<RequestSlot<Request, Self::ReplySender<Response>>>;
    /// Sends replies back to the requester. Cloned into every request
    type ReplySender<Response: Recyclable>: FeagiChannelTransmitter<ReplySlot<Response>> + Clone;
    /// Receives replies on the requester's side
    type ReplyReceiver<Response: Recyclable>: FeagiChannelReceiver<ReplySlot<Response>>;
}


pub mod impl_static {
    // TODO: Disabled until thingbuf is forked / patched. thingbuf 0.1.6 only implements `Clone` for
    // `StaticSender` with its default recycle policy, so a `StaticSender` using
    // `RecyclableRecyclePolicy` cannot satisfy
    // the `ReplySender: Clone` bound. The static channel types also require `'static` items, so the
    // GAT bounds above will need `Request: 'static` / `Response: 'static` added when this is enabled
    /*
    /// Workers use statically allocated channels
    #[derive(Debug, Clone, Copy, Default)]
    pub struct FeagiWorkerChannelsStatic;
    
    impl WorkerChannelImplementation for FeagiWorkerChannelsStatic {
        type RequestSender<Request: Recyclable, Response: Recyclable> =
            thingbuf::mpsc::StaticSender<RequestSlot<Request, Self::ReplySender<Response>>, RecyclableRecyclePolicy<RequestSlot<Request, Self::ReplySender<Response>>>>;
        type ReplySender<Response: Recyclable> = thingbuf::mpsc::StaticSender<ReplySlot<Response>, RecyclableRecyclePolicy<ReplySlot<Response>>>;
        type ReplyReceiver<Response: Recyclable> = thingbuf::mpsc::StaticReceiver<ReplySlot<Response>, RecyclableRecyclePolicy<ReplySlot<Response>>>;
    }
    */
}

#[cfg(feature = "alloc")]
pub mod impl_alloc {
    use crate::parallelism::feagi_channel::recycling::Recyclable;
    use crate::parallelism::feagi_channel::request_response_channel::implementations::dynamic::{FeagiReplyReceiverAlloc, FeagiReplyTransmitterAlloc, FeagiRequestTransmitterAlloc};
    use super::FeagiWorkerChannels;
    
    /// Workers use heap allocated channels, matching `FeagiRequestResponseDynamicBuilder`
    #[derive(Debug, Clone, Copy, Default)]
    pub struct FeagiWorkerChannelsAlloc;

    impl FeagiWorkerChannels for FeagiWorkerChannelsAlloc {
        type RequestSender<Request: Recyclable, Response: Recyclable> = FeagiRequestTransmitterAlloc<Request, Response>;
        type ReplySender<Response: Recyclable> = FeagiReplyTransmitterAlloc<Response>;
        type ReplyReceiver<Response: Recyclable> = FeagiReplyReceiverAlloc<Response>;
    }
}




