use crate::channel::recycling::{Recyclable, RecyclableRecyclePolicy};
use crate::channel::request_response::channel_structs::request_channel_pooled::RequestChannelPooled;
use crate::channel::request_response::channel_structs::response_channel::RequestResponder;

#[cfg(feature = "alloc")]
pub fn create_requester_and_responder_pooled<
    RequestStruct,
    ResponseStruct,
    const REQUEST_POOL_SIZE: usize,
    const ALLOW_BEYOND_POOL: bool>
(request_queue_length: usize)
 -> (
     RequestChannelPooled<RequestStruct, ResponseStruct, REQUEST_POOL_SIZE, ALLOW_BEYOND_POOL>,
     RequestResponder<RequestStruct, ResponseStruct,>
 )
where
    RequestStruct: Send + Recyclable,
    ResponseStruct: Send + Recyclable,
{
    let requester_pair = 
        thingbuf::mpsc::with_recycle(request_queue_length, RecyclableRecyclePolicy::new());
    let requester = RequestChannelPooled::new(requester_pair.0);
    let processor = RequestResponder::new(requester_pair.1);
    (requester, processor)
}