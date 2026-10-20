use crate::parallelism::feagi_channel::base_channels::base_traits::{FeagiChannelBuilder, FeagiChannelReceiver, FeagiChannelTransmitter};
use crate::parallelism::feagi_channel::channel_error::{FeagiChannelReceiveError, FeagiChannelTransmitError};
use crate::parallelism::feagi_channel::recycling::{Recyclable, RecyclableRecyclePolicy};

/// Creates heap allocated channel pairs holding up to `capacity` items each.
pub struct FeagiChannelAllocBuilder<T: Recyclable> {
    capacity: usize,
    _item: core::marker::PhantomData<fn() -> T>,
}

impl<T: Recyclable> FeagiChannelAllocBuilder<T> {
    pub const fn new(capacity: usize) -> Self {
        FeagiChannelAllocBuilder {
            capacity,
            _item: core::marker::PhantomData,
        }
    }

    /// Creates a new, independent transmitter and receiver. Can be called any number of times.
    /// Clone the transmitter for multiple producers
    pub fn create_channels(&self) -> (thingbuf::mpsc::Sender<T, RecyclableRecyclePolicy<T>>, thingbuf::mpsc::Receiver<T, RecyclableRecyclePolicy<T>>) {
        thingbuf::mpsc::with_recycle(self.capacity, RecyclableRecyclePolicy::<T>::new())
    }
}

impl<T: Recyclable> FeagiChannelBuilder<T> for FeagiChannelAllocBuilder<T> {
    type Transmitter = thingbuf::mpsc::Sender<T, RecyclableRecyclePolicy<T>>;
    type Receiver = thingbuf::mpsc::Receiver<T, RecyclableRecyclePolicy<T>>;
}

impl<T: Recyclable> FeagiChannelTransmitter<T> for thingbuf::mpsc::Sender<T, RecyclableRecyclePolicy<T>> {
    type SendGuard<'a>
    = thingbuf::mpsc::SendRef<'a, T>
    where
        Self: 'a;

    async fn async_send(&self, value: T) -> Result<(), FeagiChannelTransmitError> {
        thingbuf::mpsc::Sender::send(self, value).await.map_err(|e| e.into())
    }

    fn try_send(&self, value: T) -> Result<(), FeagiChannelTransmitError> {
        thingbuf::mpsc::Sender::try_send(self, value).map_err(|e| e.into())
    }

    async fn async_send_ref(&self) -> Result<Self::SendGuard<'_>, FeagiChannelTransmitError> {
        thingbuf::mpsc::Sender::send_ref(self).await.map_err(|e| e.into())
    }

    fn try_send_ref(&self) -> Result<Self::SendGuard<'_>, FeagiChannelTransmitError> {
        thingbuf::mpsc::Sender::try_send_ref(self).map_err(|e| e.into())
    }

    fn capacity(&self) -> usize {
        thingbuf::mpsc::Sender::capacity(self)
    }

    fn remaining(&self) -> usize {
        thingbuf::mpsc::Sender::remaining(self)
    }

    fn len(&self) -> usize {
        thingbuf::mpsc::Sender::len(self)
    }

    fn is_empty(&self) -> bool {
        thingbuf::mpsc::Sender::is_empty(self)
    }

    fn is_full(&self) -> bool {
        thingbuf::mpsc::Sender::remaining(self) == 0
    }
}

impl<T: Recyclable> FeagiChannelReceiver<T> for thingbuf::mpsc::Receiver<T, RecyclableRecyclePolicy<T>> {
    type RecvGuard<'a>
    = thingbuf::mpsc::RecvRef<'a, T>
    where
        Self: 'a;

    async fn async_recv(&self) -> Result<T, FeagiChannelReceiveError> {
        thingbuf::mpsc::Receiver::recv(self).await.ok_or(FeagiChannelReceiveError::ChannelClosed)
    }

    fn try_recv(&self) -> Result<T, FeagiChannelReceiveError> {
        thingbuf::mpsc::Receiver::try_recv(self).map_err(|e| e.into())
    }

    async fn async_recv_ref(&self) -> Result<Self::RecvGuard<'_>, FeagiChannelReceiveError> {
        thingbuf::mpsc::Receiver::recv_ref(self).await.ok_or(FeagiChannelReceiveError::ChannelClosed)
    }

    fn try_recv_ref(&self) -> Result<Self::RecvGuard<'_>, FeagiChannelReceiveError> {
        thingbuf::mpsc::Receiver::try_recv_ref(self).map_err(|e| e.into())
    }

    fn capacity(&self) -> usize {
        thingbuf::mpsc::Receiver::capacity(self)
    }

    fn remaining(&self) -> usize {
        thingbuf::mpsc::Receiver::remaining(self)
    }

    fn len(&self) -> usize {
        thingbuf::mpsc::Receiver::len(self)
    }

    fn is_empty(&self) -> bool {
        thingbuf::mpsc::Receiver::is_empty(self)
    }

    fn is_full(&self) -> bool {
        thingbuf::mpsc::Receiver::remaining(self) == 0
    }

    // TODO: Disabled, see `FeagiChannelReceiver`
    // fn is_closed(&self) -> bool {
    //     thingbuf::mpsc::Receiver::is_closed(self)
    // }
}