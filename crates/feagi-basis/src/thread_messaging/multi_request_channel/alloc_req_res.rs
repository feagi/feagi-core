use std::error::Error;
use thingbuf::mpsc::{Receiver, Sender};
use thingbuf::mpsc::errors::TryRecvError;
use crate::blocking_pool::BlockingPool;
use crate::thread_messaging::errors::{ChannelError, FeagiFailChannelClosed, FeagiFailPoolEmpty, FeagiFailPoolFull};

pub fn create_requester_and_processor<Req, Res, ResRec, ReqRec, ResErr, const REQUEST_POOL_SIZE: usize, const ALLOW_BEYOND_POOL: bool>
(request_queue_length: usize, request_recycler: ReqRec, response_recycler: ResRec)
    -> (
    PooledOneshotRequester<Req, Res, ResRec, ReqRec, ResErr, REQUEST_POOL_SIZE, ALLOW_BEYOND_POOL>,
    RequestProcessor<Req, Res, ResRec, ReqRec, ResErr>
    )
where
    Req: Send,
    Res: Send,
    ResRec: thingbuf::Recycle<Result<Res, ResErr>> + Clone,
    ReqRec: thingbuf::Recycle<(Req, Sender<Result<Res, ResErr>, ResRec>)> + Clone,
    ResErr: Error + Send
{
    let requester_pair = thingbuf::mpsc::with_recycle(request_queue_length, request_recycler);
    let requester = PooledOneshotRequester::new(requester_pair.0, response_recycler);
    let processor = RequestProcessor::new(requester_pair.1);
    (requester, processor)
}


pub struct PooledOneshotRequester<Req, Res, ResRec, ReqRec, ResErr, const POOL_SIZE: usize, const ALLOW_BEYOND_POOL: bool>
where
    Req: Send,
    Res: Send,
    ResRec: thingbuf::Recycle<Result<Res, ResErr>> + Clone,
    ReqRec: thingbuf::Recycle<(Req, Sender<Result<Res, ResErr>, ResRec>)> + Clone,
    ResErr: Error + Send
{
    oneshot_response_pool: BlockingPool<(Sender<Result<Res, ResErr>, ResRec>, Receiver<Result<Res, ResErr>, ResRec>), POOL_SIZE>,
    request_sender: Sender<(Req, Sender<Result<Res, ResErr>, ResRec>), ReqRec>,
    recycle_policy: ResRec,
    _p: core::marker::PhantomData<ResErr>
}

impl<Req, Res, ResRec, ReqRec, ResErr, const POOL_SIZE: usize, const ALLOW_BEYOND_POOL: bool>
PooledOneshotRequester<Req, Res, ResRec, ReqRec, ResErr, POOL_SIZE, ALLOW_BEYOND_POOL>
where
    Req: Send,
    Res: Send,
    ResRec: thingbuf::Recycle<Result<Res, ResErr>> + Clone,
    ReqRec: thingbuf::Recycle<(Req, Sender<Result<Res, ResErr>, ResRec>)> + Clone,
    ResErr: Error + Send
{
    /// Creates a new Requester with an internal channel system to the Request Processor and pool for the oneshots
    fn new(request_sender: Sender<(Req, Sender<Result<Res, ResErr>, ResRec>), ReqRec>, response_recycle_policy: ResRec) -> Self {
        let mut pool: heapless::Vec<(Sender<Result<Res, ResErr>, ResRec>, Receiver<Result<Res, ResErr>, ResRec>),POOL_SIZE> = heapless::Vec::new();
        for _ in 0..POOL_SIZE {
            _ = pool.push(Self::create_oneshot_pair(&response_recycle_policy)) // will not overflow
        };
        let oneshot_response_pool = BlockingPool::new(pool);
        Self {
            oneshot_response_pool,
            request_sender,
            recycle_policy: response_recycle_policy,
            _p: Default::default()
        }
    }

    /// Make a request, await it over the request channel, get a response
    pub async fn make_request(&mut self, request: Req) -> Result< Result<Res, ResErr>, ChannelError> {
        let maybe_oneshot_pair = self.oneshot_response_pool.take_item();
        let oneshot_pair;
        match maybe_oneshot_pair {
            None => {
                // If we allow taking from beyond the pool, initialize new
                if ALLOW_BEYOND_POOL {
                    oneshot_pair = Self::create_oneshot_pair(&self.recycle_policy);
                } else {
                    return Err(FeagiFailPoolEmpty::new("Too many requests pending! Unable to make another request!").into())
                }
            }
            Some(op) => {
                oneshot_pair = op;
            }
        };
        let oneshot_sender_clone = oneshot_pair.0.clone();
        self.request_sender.send((request, oneshot_sender_clone)).await?;

        let response = oneshot_pair.1.recv().await;
        let response = match response {
            None => {
                return Err(
                    FeagiFailChannelClosed::new("Oneshot sender died!").into()
                )
            }
            Some(r) => {r}
        };
        
        // we do need to return our oneshot channel to the pool. If the pool is full, we dont care
        _ = self.oneshot_response_pool.return_item(oneshot_pair);
        Ok(response)
    }
    
    
    fn create_oneshot_pair(recycle_policy: &ResRec) -> (Sender<Result<Res, ResErr>, ResRec>, Receiver<Result<Res, ResErr>, ResRec>) {
        let channel = thingbuf::mpsc::with_recycle(1, recycle_policy.clone());
        channel
    }
}

impl<Req, Res, ResRec, ReqRec, ResErr, const POOL_SIZE: usize, const ALLOW_BEYOND_POOL: bool>
Clone for
PooledOneshotRequester<Req, Res, ResRec, ReqRec, ResErr, POOL_SIZE, ALLOW_BEYOND_POOL>
where
    Req: Send,
    Res: Send,
    ResRec: thingbuf::Recycle<Result<Res, ResErr>> + Clone,
    ReqRec: thingbuf::Recycle<(Req, Sender<Result<Res, ResErr>, ResRec>)> + Clone,
    ResErr: Error + Send
{
    fn clone(&self) -> Self {
        Self::new(
            self.request_sender.clone(),
            self.recycle_policy.clone()
        )
    }
}

// NOTE: Do NOT allow this to be clonable!
/// Allows processing of multiple incoming datas and sending responses via oneshots in an MPSC
/// pattern. Call 'loop_process_incoming_requests' in a  loop to poll for incoming requests,
/// process them, and then return the result
pub struct RequestProcessor<Req, Res, ResRec, ReqRec, ResErr>
where
    Req: Send,
    Res: Send,
    ResRec: thingbuf::Recycle<Result<Res, ResErr>> + Clone,
    ReqRec: thingbuf::Recycle<(Req, Sender<Result<Res, ResErr>, ResRec>)> + Clone,
    ResErr: Error + Send
{
    incoming_requests: Receiver<(Req, Sender<Result<Res, ResErr>, ResRec>), ReqRec>,
    // outgoing responses go over oneshot
}


impl<Req, Res, ResRec, ReqRec, ResErr> RequestProcessor<Req, Res, ResRec, ReqRec, ResErr>
where
    Req: Send,
    Res: Send,
    ResRec: thingbuf::Recycle<Result<Res, ResErr>> + Clone,
    ReqRec: thingbuf::Recycle<(Req, Sender<Result<Res, ResErr>, ResRec>)> + Clone,
    ResErr: Error + Send
{

    fn new(incoming_requests: Receiver<(Req, Sender<Result<Res, ResErr>, ResRec>), ReqRec>) -> Self {
        Self {incoming_requests}
    }

    /*
    pub async fn loop_process_incoming_requests_mut<ProcessorFuncMut>(&self, mut processor_func: ProcessorFuncMut) -> Result<(), ChannelError>
    where
        ProcessorFuncMut: AsyncFnMut(Req) -> Result<Res, ResErr>
    {
        self.loop_process_incoming_requests::<ProcessorFuncMut>(&self, processor_func).await
    }
     */

    pub async fn loop_process_incoming_requests<ProcessorFunc>(&self, processor_func: ProcessorFunc) -> Result<(), ChannelError>
    where
        ProcessorFunc: AsyncFn(Req) -> Result<Res, ResErr>
    {
        let maybe_incoming = self.incoming_requests.try_recv();

        match maybe_incoming {
            Err(e) => {
                match e {
                    TryRecvError::Empty => {
                        return Ok(()) // nothing to do, return
                    }
                    TryRecvError::Closed => {
                        return Err(FeagiFailChannelClosed::new("Request Receiver channels have all be closed!").into())
                    }
                    _ => {
                        // For some reason the linter is forcing this, even though this is impossible??
                        unreachable!("This enum only has 2 errors?")
                    }
                }
            }
            Ok(incoming) => {
                let response = processor_func(incoming.0).await;
                let sender = incoming.1;
                _ = sender.send(response).await; // We do not care if the oneshot is closed
                drop(sender); // to be absolutely clear, destroy the oneshot returner
                Ok(())
            }
        }


    }


}
