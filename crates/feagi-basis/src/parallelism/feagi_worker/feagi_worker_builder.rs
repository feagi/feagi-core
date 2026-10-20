use crate::parallelism::feagi_channel::feagi_channel::{FeagiChannelReceiver, FeagiChannelSender};
use crate::parallelism::feagi_channel::recycling::Recyclable;
#[cfg(feature = "alloc")]
use crate::parallelism::feagi_channel::request_response::feagi_request_response::{FeagiReplyReceiverDynamic, FeagiReplySenderDynamic, FeagiRequestSenderDynamic};
use crate::parallelism::feagi_channel::request_response::feagi_request_response::{ReplySlot, RequestSlot};
use crate::parallelism::feagi_worker::feagi_worker::FeagiWorker;
use crate::parallelism::feagi_worker::worker_spawner_implementation::WorkerSpawnerImplementation;

/// Handles the construction of various 'FeagiWorker' types
pub trait FeagiWorkerBuilder<Spawner: WorkerSpawnerImplementation> {
    /// Which channel types (heap allocated, static) the built worker uses for its requests
    type Channels: WorkerChannelImplementation;
    /// What the builder needs to build the worker, including any incoming request receiver
    /// channels from `spawn_required_channel_pairs`
    type BuildInputs;
    /// What the builder will output when building the worker (in addition to the worker)
    type BuildOutputs;
    /// Workers often have additional channels going in or out of themselves. This defines those
    type RequiredChannelPairs;
    /// What type of custom request going in is supported
    type CustomRequest;
    /// What type of custom response coming out is supported
    type CustomResponse;

    type BuildError: core::error::Error;

    /// Consume self to build the FeagiWorker
    fn build_worker(
        self,
        spawner: &Spawner,
        inputs: Self::BuildInputs,
    ) -> Result<
        (
            FeagiWorker<Spawner, Self::Channels, Self::CustomRequest, Self::CustomResponse>,
            Self::BuildOutputs,
        ),
        Self::BuildError,
    >;

    /// Create needed channel pairings. Done first so different workers can be aligned before
    /// building them
    fn spawn_required_channel_pairs(&self) -> Self::RequiredChannelPairs;
}

/// Which channel types a worker uses to receive requests and send replies (heap allocated,
/// static)
pub trait WorkerChannelImplementation {
    /// Sends requests, each carrying a copy of the requester's reply sender
    type RequestSender<Request: Recyclable, Response: Recyclable>: FeagiChannelSender<RequestSlot<Request, Self::ReplySender<Response>>>;
    /// Sends replies back to the requester. Cloned into every request
    type ReplySender<Response: Recyclable>: FeagiChannelSender<ReplySlot<Response>> + Clone;
    /// Receives replies on the requester's side
    type ReplyReceiver<Response: Recyclable>: FeagiChannelReceiver<ReplySlot<Response>>;
}

//region Dynamic Channels

/// Workers use heap allocated channels, matching `FeagiRequestResponseDynamicBuilder`
#[cfg(feature = "alloc")]
#[derive(Debug, Clone, Copy, Default)]
pub struct FeagiWorkerChannelsDynamic;

#[cfg(feature = "alloc")]
impl WorkerChannelImplementation for FeagiWorkerChannelsDynamic {
    type RequestSender<Request: Recyclable, Response: Recyclable> = FeagiRequestSenderDynamic<Request, Response>;
    type ReplySender<Response: Recyclable> = FeagiReplySenderDynamic<Response>;
    type ReplyReceiver<Response: Recyclable> = FeagiReplyReceiverDynamic<Response>;
}

//endregion

//region Static Channels

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

//endregion
