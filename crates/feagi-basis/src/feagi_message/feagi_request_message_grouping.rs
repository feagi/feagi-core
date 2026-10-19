use crate::feagi_message::feagi_request_message::FeagiRequestMessageDescriptor;
use crate::parallelism::feagi_channel::feagi_channel::{FeagiChannelReceiver, FeagiChannelSender};
use crate::parallelism::feagi_channel::recycling::Recyclable;
use crate::parallelism::feagi_channel::request_response::feagi_request_response::{FeagiRequestResponseBuilder, FeagiRequester, ReplySlot, RequestSlot};

pub trait RequestCategoryChannelRouter<Request, Response, ReplySender, RequestSender, ReplyReceiver>
where
    Request: Recyclable,
    Response: Recyclable,
    ReplySender: FeagiChannelSender<ReplySlot<Response>> + Clone,
    RequestSender: FeagiChannelSender<RequestSlot<Request, ReplySender>>,
    ReplyReceiver: FeagiChannelReceiver<ReplySlot<Response>>,
{
    
    // Assumption that the requests implement Into<RequestEnum> (same for Response)
    
    /// Tuple of all Feagi Request Message Descriptors in this category
    type ContainedMessagesRecursiveTuple: RecursiveTupleLayer;
}

pub trait RecursiveTupleLayer {
    type Output;
    
    fn process_self(self) -> Self::Output;
    
}

impl<X, Y> RecursiveTupleLayer for (X, Y)
where X: FeagiRequestMessageDescriptor
{
    type Output = Y;

    fn process_self(self) -> Self::Output {
        todo!()
    }
}



