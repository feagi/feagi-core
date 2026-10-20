use crate::parallelism::feagi_channel::channel_error::{FeagiChannelReceiveError, FeagiChannelSendError};
use crate::parallelism::feagi_channel::recycling::{Recyclable, RecyclableRecyclePolicy};
use core::ops::DerefMut;

// NOTE: These traits are implemented directly on thingbuf's channel halves. They exist so code
// can be generic over heap allocated and static channels, which thingbuf has no common trait for

// TODO optional timeout policy as a generic?

/// Marks a type that builds feagi channels carrying T
pub trait FeagiChannelBuilder<T: Recyclable> {
    /// Sending half produced by this builder
    type Sender: FeagiChannelSender<T>;
    /// Receiving half produced by this builder
    type Receiver: FeagiChannelReceiver<T>;
}

/// Sending half of a feagi channel
pub trait FeagiChannelSender<T: Recyclable> {
    /// Guard over a claimed slot, written in place. The slot is published to the receiver when
    /// the guard drops, even if nothing was written, so keep it short-lived and never hold it
    /// across an await point
    type SendGuard<'a>: DerefMut<Target = T>
    where
        Self: 'a;

    /// Waits for capacity, then sends. The value is dropped if the channel is closed
    async fn async_send(&self, value: T) -> Result<(), FeagiChannelSendError>;
    /// Sends without waiting. The value is dropped if the channel is full or closed
    fn try_send(&self, value: T) -> Result<(), FeagiChannelSendError>;
    /// Waits for capacity, then claims a slot holding a recycled value to write into
    async fn async_send_ref(&self) -> Result<Self::SendGuard<'_>, FeagiChannelSendError>;
    /// Claims a slot holding a recycled value to write into without waiting
    fn try_send_ref(&self) -> Result<Self::SendGuard<'_>, FeagiChannelSendError>;
    /// Total number of items the channel can hold
    fn capacity(&self) -> usize;
    /// Number of items that can still be sent before the channel is full
    fn remaining(&self) -> usize;
    /// Number of items currently queued in the channel
    fn len(&self) -> usize;
    /// True if no items are currently queued in the channel
    fn is_empty(&self) -> bool;
    /// True if no more items can be sent until the receiver takes some
    fn is_full(&self) -> bool;
}

/// Receiving half of a feagi channel.
pub trait FeagiChannelReceiver<T: Recyclable> {
    /// Guard over a received slot, read in place. The slot is released back to the channel
    /// for recycling when the guard drops, so keep it short-lived and never hold it across an
    /// await point
    type RecvGuard<'a>: DerefMut<Target = T>
    where
        Self: 'a;

    /// Waits for the next value. Errors with `ChannelClosed` once every sender is gone and the
    /// channel is drained
    async fn async_recv(&self) -> Result<T, FeagiChannelReceiveError>;
    /// Returns immediately with a value, or reports empty or closed
    fn try_recv(&self) -> Result<T, FeagiChannelReceiveError>;
    /// Waits for the next value and borrows it in place instead of moving it out
    async fn async_recv_ref(&self) -> Result<Self::RecvGuard<'_>, FeagiChannelReceiveError>;
    /// Borrows the next value in place without waiting, or reports empty or closed
    fn try_recv_ref(&self) -> Result<Self::RecvGuard<'_>, FeagiChannelReceiveError>;
    /// Total number of items the channel can hold
    fn capacity(&self) -> usize;
    /// Number of items that can still be sent before the channel is full
    fn remaining(&self) -> usize;
    /// Number of items currently queued in the channel
    fn len(&self) -> usize;
    /// True if no items are currently queued in the channel
    fn is_empty(&self) -> bool;
    /// True if no more items can be sent until this receiver takes some
    fn is_full(&self) -> bool;
    // TODO: Disabled because thingbuf 0.1.6 `Receiver::is_closed` reports closed while one sender
    // is still alive (it checks `tx_count <= 1`, but the count starts at 1 for the first sender).
    // Restore once thingbuf is fixed or patched. Receiving still reports `ChannelClosed` correctly
    // /// True if every sender has been dropped. Items already queued can still be received
    // fn is_closed(&self) -> bool;
}


//region Static Channels

// TODO: Disabled until thingbuf can construct a StaticChannel with a custom recycle policy.
// thingbuf 0.1.6 only provides `StaticChannel::new()` for its default policy
/*
pub struct FeagiChannelStaticBuilder<T: Recyclable + 'static, const CAPACITY: usize> (thingbuf::mpsc::StaticChannel<T, CAPACITY, RecyclableRecyclePolicy<T>>);

impl<T: Recyclable + 'static, const CAPACITY: usize> FeagiChannelStaticBuilder<T, CAPACITY> {
    pub const fn new() -> Self {
        FeagiChannelStaticBuilder (
            thingbuf::mpsc::StaticChannel::<T, CAPACITY, RecyclableRecyclePolicy<T>>::new()
        )
    }

    /// Splits the channel into its sender and receiver
    /// A static channel can only be split once.
    pub fn create_channels(&'static self) -> (thingbuf::mpsc::StaticSender<T, RecyclableRecyclePolicy<T>>, thingbuf::mpsc::StaticReceiver<T, RecyclableRecyclePolicy<T>>) {
        self.0.split()
    }
}

impl<T: Recyclable + 'static, const CAPACITY: usize> FeagiChannelBuilder<T> for FeagiChannelStaticBuilder<T, CAPACITY> {
    type Sender = thingbuf::mpsc::StaticSender<T, RecyclableRecyclePolicy<T>>;
    type Receiver = thingbuf::mpsc::StaticReceiver<T, RecyclableRecyclePolicy<T>>;
}
*/

impl<T: Recyclable + 'static> FeagiChannelSender<T> for thingbuf::mpsc::StaticSender<T, RecyclableRecyclePolicy<T>> {
    type SendGuard<'a>
        = thingbuf::mpsc::SendRef<'a, T>
    where
        Self: 'a;

    async fn async_send(&self, value: T) -> Result<(), FeagiChannelSendError> {
        thingbuf::mpsc::StaticSender::send(self, value).await.map_err(|e| e.into())
    }

    fn try_send(&self, value: T) -> Result<(), FeagiChannelSendError> {
        thingbuf::mpsc::StaticSender::try_send(self, value).map_err(|e| e.into())
    }

    async fn async_send_ref(&self) -> Result<Self::SendGuard<'_>, FeagiChannelSendError> {
        thingbuf::mpsc::StaticSender::send_ref(self).await.map_err(|e| e.into())
    }

    fn try_send_ref(&self) -> Result<Self::SendGuard<'_>, FeagiChannelSendError> {
        thingbuf::mpsc::StaticSender::try_send_ref(self).map_err(|e| e.into())
    }

    fn capacity(&self) -> usize {
        thingbuf::mpsc::StaticSender::capacity(self)
    }

    fn remaining(&self) -> usize {
        thingbuf::mpsc::StaticSender::remaining(self)
    }

    fn len(&self) -> usize {
        thingbuf::mpsc::StaticSender::len(self)
    }

    fn is_empty(&self) -> bool {
        thingbuf::mpsc::StaticSender::is_empty(self)
    }

    fn is_full(&self) -> bool {
        thingbuf::mpsc::StaticSender::remaining(self) == 0
    }
}

impl<T: Recyclable + 'static> FeagiChannelReceiver<T> for thingbuf::mpsc::StaticReceiver<T, RecyclableRecyclePolicy<T>> {
    type RecvGuard<'a>
        = thingbuf::mpsc::RecvRef<'a, T>
    where
        Self: 'a;

    async fn async_recv(&self) -> Result<T, FeagiChannelReceiveError> {
        thingbuf::mpsc::StaticReceiver::recv(self).await.ok_or(FeagiChannelReceiveError::ChannelClosed)
    }

    fn try_recv(&self) -> Result<T, FeagiChannelReceiveError> {
        thingbuf::mpsc::StaticReceiver::try_recv(self).map_err(|e| e.into())
    }

    async fn async_recv_ref(&self) -> Result<Self::RecvGuard<'_>, FeagiChannelReceiveError> {
        thingbuf::mpsc::StaticReceiver::recv_ref(self).await.ok_or(FeagiChannelReceiveError::ChannelClosed)
    }

    fn try_recv_ref(&self) -> Result<Self::RecvGuard<'_>, FeagiChannelReceiveError> {
        thingbuf::mpsc::StaticReceiver::try_recv_ref(self).map_err(|e| e.into())
    }

    fn capacity(&self) -> usize {
        thingbuf::mpsc::StaticReceiver::capacity(self)
    }

    fn remaining(&self) -> usize {
        thingbuf::mpsc::StaticReceiver::remaining(self)
    }

    fn len(&self) -> usize {
        thingbuf::mpsc::StaticReceiver::len(self)
    }

    fn is_empty(&self) -> bool {
        thingbuf::mpsc::StaticReceiver::is_empty(self)
    }

    fn is_full(&self) -> bool {
        thingbuf::mpsc::StaticReceiver::remaining(self) == 0
    }

    // TODO: Disabled, see `FeagiChannelReceiver`
    // fn is_closed(&self) -> bool {
    //     thingbuf::mpsc::StaticReceiver::is_closed(self)
    // }
}

//endregion

//region Alloc Channels

/// Creates heap allocated channel pairs holding up to `capacity` items each.
#[cfg(feature = "alloc")]
pub struct FeagiChannelAllocBuilder<T: Recyclable> {
    capacity: usize,
    _item: core::marker::PhantomData<fn() -> T>,
}

#[cfg(feature = "alloc")]
impl<T: Recyclable> FeagiChannelAllocBuilder<T> {
    pub const fn new(capacity: usize) -> Self {
        FeagiChannelAllocBuilder {
            capacity,
            _item: core::marker::PhantomData,
        }
    }

    /// Creates a new, independent sender and receiver. Can be called any number of times.
    /// Clone the sender for multiple producers
    pub fn create_channels(&self) -> (thingbuf::mpsc::Sender<T, RecyclableRecyclePolicy<T>>, thingbuf::mpsc::Receiver<T, RecyclableRecyclePolicy<T>>) {
        thingbuf::mpsc::with_recycle(self.capacity, RecyclableRecyclePolicy::<T>::new())
    }
}

#[cfg(feature = "alloc")]
impl<T: Recyclable> FeagiChannelBuilder<T> for FeagiChannelAllocBuilder<T> {
    type Sender = thingbuf::mpsc::Sender<T, RecyclableRecyclePolicy<T>>;
    type Receiver = thingbuf::mpsc::Receiver<T, RecyclableRecyclePolicy<T>>;
}

#[cfg(feature = "alloc")]
impl<T: Recyclable> FeagiChannelSender<T> for thingbuf::mpsc::Sender<T, RecyclableRecyclePolicy<T>> {
    type SendGuard<'a>
        = thingbuf::mpsc::SendRef<'a, T>
    where
        Self: 'a;

    async fn async_send(&self, value: T) -> Result<(), FeagiChannelSendError> {
        thingbuf::mpsc::Sender::send(self, value).await.map_err(|e| e.into())
    }

    fn try_send(&self, value: T) -> Result<(), FeagiChannelSendError> {
        thingbuf::mpsc::Sender::try_send(self, value).map_err(|e| e.into())
    }

    async fn async_send_ref(&self) -> Result<Self::SendGuard<'_>, FeagiChannelSendError> {
        thingbuf::mpsc::Sender::send_ref(self).await.map_err(|e| e.into())
    }

    fn try_send_ref(&self) -> Result<Self::SendGuard<'_>, FeagiChannelSendError> {
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

#[cfg(feature = "alloc")]
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

//endregion
