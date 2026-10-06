use thingbuf::mpsc::{Receiver, Sender};
use thingbuf::mpsc::errors::TryRecvError;
use crate::collections::generic_collections::blocking_pool::BlockingPool;
use crate::threading::thread_messaging::ChannelError;




pub fn create_requester_and_responder<Req, Res, ResRec, ReqRec, const REQUEST_POOL_SIZE: usize, const ALLOW_BEYOND_POOL: bool>
(request_queue_length: usize, request_recycler: ReqRec, response_recycler: ResRec)
    -> (
        RequestChannelPooled<Req, Res, ResRec, ReqRec, REQUEST_POOL_SIZE, ALLOW_BEYOND_POOL>,
        RequestResponder<Req, Res, ResRec, ReqRec, >
    )
where
    Req: Send,
    Res: Send,
    ResRec: thingbuf::Recycle<Res> + Clone,
    ReqRec: thingbuf::Recycle<(Req, Sender<Res, ResRec>)> + Clone
{
    let requester_pair = thingbuf::mpsc::with_recycle(request_queue_length, request_recycler);
    let requester = RequestChannelPooled::new(requester_pair.0, response_recycler);
    let processor = RequestResponder::new(requester_pair.1);
    (requester, processor)

}




//region ResponderChannel



//endregion

//region Recycling


//endregion