use crate::blocking_pool::BlockingPool;

/// A marker trait used to quickly generating oneshot transmitters and receivers 
pub trait PooledOneshotPair<Res: Send> {
    type Error;
    type Tx: PooledOneshotTransmitterChannel<Res, Error=Self::Error>;
    type Rx: PooledOneshotReceiverChannel<Res, Error=Self::Error>;

    /// Creates a new pair of oneshot transmitters and receivers for returning responses
    fn new_pair() -> (Self::Tx, Self::Rx);
}

/// Is cloned and sent to the request processor for it to send the response back in
pub trait PooledOneshotTransmitterChannel<Res: Send>: Send + Clone {
    type Error;

    /// Send the response through here then toss out
    async fn return_response_through_oneshot(&mut self, response: Res);
}

pub trait PooledOneshotReceiverChannel<Res: Send> {
    type Error;

    async fn await_response_through_oneshot(&mut self) -> Res;
}



pub struct PooledOneshotRequester<Req, Res, Pair, const POOL_SIZE: usize>
where
    Req: Send,
    Res: Send,
    Pair: PooledOneshotPair<Res>
{
    pool: BlockingPool<(Pair::Tx, Pair::Rx), POOL_SIZE>,
    _p: core::marker::PhantomData<Req>
}

impl<Req, Res, Pair, const POOL_SIZE: usize> PooledOneshotRequester<Req, Res, Pair, POOL_SIZE>
where
    Req: Send,
    Res: Send,
    Pair: PooledOneshotPair<Res>
{
    pub fn new() -> Self {
        let mut pool: heapless::Vec<(Pair::Tx, Pair::Rx),POOL_SIZE> = heapless::Vec::new();
        for _ in 0..POOL_SIZE {
            _ = pool.push(Pair::new_pair()) // will not overflow
        };
        let pool = BlockingPool::new(pool);
        Self {pool, _p: Default::default()}
    }
}