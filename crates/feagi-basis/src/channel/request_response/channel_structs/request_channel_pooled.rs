use thingbuf::mpsc::{Receiver, Sender};
use crate::channel::channel_error::ChannelError;
use crate::channel::recycling::{Recyclable, RecyclableRecyclePolicy};
use crate::threading::BlockingPool;

type OneShot<Request, Response> = (Request, Sender<Response, RecyclableRecyclePolicy<Response>>);

/// Allows sending requests to a response channel. This is clonable, allowing multiple requestors to
/// request from a single response channel (MPSC). This variant uses request pooling, which stores
/// the oneshot responses in a buffer to avoid repeated allocations. However, the bool flag
/// allows the struct to allocate additional oneshots if it exceeds the request pool
pub struct RequestChannelPooled<Request, Response, const POOL_SIZE: usize, const ALLOW_BEYOND_POOL: bool>
where
    Request: Send + Recyclable,
    Response: Send + Recyclable,
{
    oneshot_response_pool: BlockingPool<(
        Sender<Response, RecyclableRecyclePolicy<Response>>,
        Receiver<Response, RecyclableRecyclePolicy<Response>>),
        POOL_SIZE>,
    request_sender: Sender<OneShot<Request, Response>, RecyclableRecyclePolicy<OneShot<Request, Response>>>
}

impl<Request, Response, const POOL_SIZE: usize, const ALLOW_BEYOND_POOL: bool>
RequestChannelPooled<Request, Response, POOL_SIZE, ALLOW_BEYOND_POOL>
where
    Request: Send + Recyclable,
    Response: Send + Recyclable,
{
    /// Creates a new Requester with an internal channel system to the Request Processor and pool for the oneshots
    pub(crate) fn new(request_sender: Sender<OneShot<Request, Response>, RecyclableRecyclePolicy<OneShot<Request, Response>>>) -> Self {

        let mut pool: heapless::Vec<
            (Sender<Response, RecyclableRecyclePolicy<Response>>,
             Receiver<Response, RecyclableRecyclePolicy<Response>>),
            POOL_SIZE
        > = heapless::Vec::new();

        let (oneshot_sender, oneshot_receiver) = Self::create_oneshot_pair();
        for _ in 0..POOL_SIZE {
            _ = pool.push(Self::create_oneshot_pair()) // will not overflow
        };
        let oneshot_response_pool = BlockingPool::new(pool);
        Self {
            oneshot_response_pool,
            request_sender,
        }
    }

    /// Make a request, await it over the request channel, get a response
    pub async fn make_request(&mut self, request: Request) -> Result<Response, ChannelError> {
        let maybe_oneshot_pair = self.oneshot_response_pool.take_item();
        let oneshot_pair;
        match maybe_oneshot_pair {
            None => {
                // If we allow taking from beyond the pool, initialize new
                if ALLOW_BEYOND_POOL {
                    oneshot_pair = Self::create_oneshot_pair();
                } else {
                    return Err(ChannelError::PoolEmpty);
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
                return Err(ChannelError::SendClosed);
            }
            Some(r) => {r}
        };

        // we do need to return our oneshot channel to the pool. If the pool is full, we dont care
        _ = self.oneshot_response_pool.return_item(oneshot_pair);
        Ok(response)
    }

    fn create_oneshot_pair() -> (Sender<Response, RecyclableRecyclePolicy<Response>>, Receiver<Response, RecyclableRecyclePolicy<Response>>) {
        let channel = thingbuf::mpsc::with_recycle(1, RecyclableRecyclePolicy::NEW);
        channel
    }
}

impl<Request, Response, const POOL_SIZE: usize, const ALLOW_BEYOND_POOL: bool> Clone for RequestChannelPooled<Request, Response, POOL_SIZE, ALLOW_BEYOND_POOL>
where
    Request: Send + Recyclable + Clone,
    Response: Send + Recyclable + Clone,
{
    fn clone(&self) -> Self {
        Self::new(self.request_sender.clone())
    }
}