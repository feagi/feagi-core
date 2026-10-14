use core::ops::DerefMut;
use crate::parallelism::feagi_channel::channel_error::{FeagiChannelReceiveError, FeagiChannelSendError};
use crate::parallelism::feagi_channel::recycling::{Recyclable, RecyclableRecyclePolicy};

// NOTE: Yes this is a bit repetitive over Thingbuf, but lets keep the upstream consistent

// TODO optional timeout policy as a generic?

/// Sending half of a feagi channel
pub trait FeagiChannelSender<T: Recyclable> {
    /// Guard over a claimed slot, written in place. The slot is published to the receiver when
    /// the guard drops, even if nothing was written, so keep it short-lived and never hold it
    /// across an await point
    type SendGuard<'a>: DerefMut<Target = T> where Self: 'a;

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
    type RecvGuard<'a>: DerefMut<Target = T> where Self: 'a;

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
    /// True if every sender has been dropped. Items already queued can still be received
    fn is_closed(&self) -> bool;
}

/// Marks a type that builds feagi channels carrying T
pub trait FeagiChannelBuilder<T: Recyclable> {
    /// Sending half produced by this builder
    type Sender: FeagiChannelSender<T>;
    /// Receiving half produced by this builder
    type Receiver: FeagiChannelReceiver<T>;
}

//region Static Channels

pub struct FeagiChannelStaticBuilder<T: Recyclable + 'static, const CAPACITY: usize> (thingbuf::mpsc::StaticChannel<T, CAPACITY, RecyclableRecyclePolicy<T>>);

impl<T: Recyclable + 'static, const CAPACITY: usize> FeagiChannelStaticBuilder<T, CAPACITY> {
    pub const fn new() -> Self {
        FeagiChannelStaticBuilder (
            thingbuf::mpsc::StaticChannel::<T, CAPACITY, RecyclableRecyclePolicy<T>>::new()
        )
    }

    /// Splits the channel into its sender and receiver
    /// A static channel can only be split once.
    pub fn create_channels(&'static self) -> (FeagiChannelSenderStatic<T>, FeagiChannelReceiverStatic<T>) {
        let (s, r) = self.0.split();
        (FeagiChannelSenderStatic(s), FeagiChannelReceiverStatic(r))
    }
}

impl<T: Recyclable + 'static, const CAPACITY: usize> FeagiChannelBuilder<T> for FeagiChannelStaticBuilder<T, CAPACITY> {
    type Sender = FeagiChannelSenderStatic<T>;
    type Receiver = FeagiChannelReceiverStatic<T>;
}

pub struct FeagiChannelSenderStatic<T: Recyclable + 'static>(thingbuf::mpsc::StaticSender<T, RecyclableRecyclePolicy<T>>);

impl<T: Recyclable + 'static> FeagiChannelSender<T> for FeagiChannelSenderStatic<T> {
    type SendGuard<'a> = thingbuf::mpsc::SendRef<'a, T> where Self: 'a;

    async fn async_send(&self, value: T) -> Result<(), FeagiChannelSendError> {
        self.0.send(value).await.map_err(|e| e.into())
    }

    fn try_send(&self, value: T) -> Result<(), FeagiChannelSendError> {
        self.0.try_send(value).map_err(|e| e.into())
    }

    async fn async_send_ref(&self) -> Result<Self::SendGuard<'_>, FeagiChannelSendError> {
        self.0.send_ref().await.map_err(|e| e.into())
    }

    fn try_send_ref(&self) -> Result<Self::SendGuard<'_>, FeagiChannelSendError> {
        self.0.try_send_ref().map_err(|e| e.into())
    }

    fn capacity(&self) -> usize {
        self.0.capacity()
    }

    fn remaining(&self) -> usize {
        self.0.remaining()
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn is_full(&self) -> bool {
        self.0.remaining() == 0
    }
}

pub struct FeagiChannelReceiverStatic<T: Recyclable + 'static>(thingbuf::mpsc::StaticReceiver<T, RecyclableRecyclePolicy<T>>);

impl<T: Recyclable + 'static> FeagiChannelReceiver<T> for FeagiChannelReceiverStatic<T> {
    type RecvGuard<'a> = thingbuf::mpsc::RecvRef<'a, T> where Self: 'a;

    async fn async_recv(&self) -> Result<T, FeagiChannelReceiveError> {
        self.0.recv().await.ok_or(FeagiChannelReceiveError::ChannelClosed)
    }

    fn try_recv(&self) -> Result<T, FeagiChannelReceiveError> {
        self.0.try_recv().map_err(|e| e.into())
    }

    async fn async_recv_ref(&self) -> Result<Self::RecvGuard<'_>, FeagiChannelReceiveError> {
        self.0.recv_ref().await.ok_or(FeagiChannelReceiveError::ChannelClosed)
    }

    fn try_recv_ref(&self) -> Result<Self::RecvGuard<'_>, FeagiChannelReceiveError> {
        self.0.try_recv_ref().map_err(|e| e.into())
    }

    fn capacity(&self) -> usize {
        self.0.capacity()
    }

    fn remaining(&self) -> usize {
        self.0.remaining()
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn is_full(&self) -> bool {
        self.0.remaining() == 0
    }

    fn is_closed(&self) -> bool {
        self.0.is_closed()
    }
}

//endregion

//region Dynamic Channels

/// Creates heap allocated channel pairs holding up to `capacity` items each.
#[cfg(feature = "alloc")]
pub struct FeagiChannelDynamicBuilder<T: Recyclable> {
    capacity: usize,
    _item: core::marker::PhantomData<fn() -> T>,
}

#[cfg(feature = "alloc")]
impl<T: Recyclable> FeagiChannelDynamicBuilder<T> {
    pub const fn new(capacity: usize) -> Self {
        FeagiChannelDynamicBuilder {
            capacity,
            _item: core::marker::PhantomData,
        }
    }

    /// Creates a new, independent sender and receiver. Can be called any number of times.
    pub fn create_channels(&self) -> (FeagiChannelSenderDynamic<T>, FeagiChannelReceiverDynamic<T>) {
        let (s, r) = thingbuf::mpsc::with_recycle(self.capacity, RecyclableRecyclePolicy::<T>::new());
        (FeagiChannelSenderDynamic(s), FeagiChannelReceiverDynamic(r))
    }
}

#[cfg(feature = "alloc")]
impl<T: Recyclable> FeagiChannelBuilder<T> for FeagiChannelDynamicBuilder<T> {
    type Sender = FeagiChannelSenderDynamic<T>;
    type Receiver = FeagiChannelReceiverDynamic<T>;
}

#[cfg(feature = "alloc")]
pub struct FeagiChannelSenderDynamic<T: Recyclable>(thingbuf::mpsc::Sender<T, RecyclableRecyclePolicy<T>>);

/// Creates another sender to the same channel, allowing multiple producers
#[cfg(feature = "alloc")]
impl<T: Recyclable> Clone for FeagiChannelSenderDynamic<T> {
    fn clone(&self) -> Self {
        FeagiChannelSenderDynamic(self.0.clone())
    }
}

#[cfg(feature = "alloc")]
impl<T: Recyclable> FeagiChannelSender<T> for FeagiChannelSenderDynamic<T> {
    type SendGuard<'a> = thingbuf::mpsc::SendRef<'a, T> where Self: 'a;

    async fn async_send(&self, value: T) -> Result<(), FeagiChannelSendError> {
        self.0.send(value).await.map_err(|e| e.into())
    }

    fn try_send(&self, value: T) -> Result<(), FeagiChannelSendError> {
        self.0.try_send(value).map_err(|e| e.into())
    }

    async fn async_send_ref(&self) -> Result<Self::SendGuard<'_>, FeagiChannelSendError> {
        self.0.send_ref().await.map_err(|e| e.into())
    }

    fn try_send_ref(&self) -> Result<Self::SendGuard<'_>, FeagiChannelSendError> {
        self.0.try_send_ref().map_err(|e| e.into())
    }

    fn capacity(&self) -> usize {
        self.0.capacity()
    }

    fn remaining(&self) -> usize {
        self.0.remaining()
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn is_full(&self) -> bool {
        self.0.remaining() == 0
    }
}

#[cfg(feature = "alloc")]
pub struct FeagiChannelReceiverDynamic<T: Recyclable>(thingbuf::mpsc::Receiver<T, RecyclableRecyclePolicy<T>>);

#[cfg(feature = "alloc")]
impl<T: Recyclable> FeagiChannelReceiver<T> for FeagiChannelReceiverDynamic<T> {
    type RecvGuard<'a> = thingbuf::mpsc::RecvRef<'a, T> where Self: 'a;

    async fn async_recv(&self) -> Result<T, FeagiChannelReceiveError> {
        self.0.recv().await.ok_or(FeagiChannelReceiveError::ChannelClosed)
    }

    fn try_recv(&self) -> Result<T, FeagiChannelReceiveError> {
        self.0.try_recv().map_err(|e| e.into())
    }

    async fn async_recv_ref(&self) -> Result<Self::RecvGuard<'_>, FeagiChannelReceiveError> {
        self.0.recv_ref().await.ok_or(FeagiChannelReceiveError::ChannelClosed)
    }

    fn try_recv_ref(&self) -> Result<Self::RecvGuard<'_>, FeagiChannelReceiveError> {
        self.0.try_recv_ref().map_err(|e| e.into())
    }

    fn capacity(&self) -> usize {
        self.0.capacity()
    }

    fn remaining(&self) -> usize {
        self.0.remaining()
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn is_full(&self) -> bool {
        self.0.remaining() == 0
    }

    fn is_closed(&self) -> bool {
        self.0.is_closed()
    }
}

//endregion
