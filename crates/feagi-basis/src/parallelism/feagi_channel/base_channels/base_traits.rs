use core::ops::DerefMut;
use crate::parallelism::feagi_channel::channel_error::{FeagiChannelReceiveError, FeagiChannelTransmitError};
use crate::parallelism::feagi_channel::recycling::Recyclable;

// NOTE: These traits are implemented directly on thingbuf's channel halves. They exist so code
// can be generic over heap allocated and static channels, which thingbuf has no common trait for

/// Marks a type that builds feagi channels carrying T
pub trait FeagiChannelBuilder<T: Recyclable> {
    /// Sending half produced by this builder
    type Transmitter: FeagiChannelTransmitter<T>;
    /// Receiving half produced by this builder
    type Receiver: FeagiChannelReceiver<T>;
}

/// Sending half of a feagi channel
pub trait FeagiChannelTransmitter<T: Recyclable> {
    /// Guard over a claimed slot, written in place. The slot is published to the receiver when
    /// the guard drops, even if nothing was written, so keep it short-lived and never hold it
    /// across an await point
    type SendGuard<'a>: DerefMut<Target = T>
    where
        Self: 'a;

    /// Waits for capacity, then sends. The value is dropped if the channel is closed
    async fn async_send(&self, value: T) -> Result<(), FeagiChannelTransmitError>;
    /// Sends without waiting. The value is dropped if the channel is full or closed
    fn try_send(&self, value: T) -> Result<(), FeagiChannelTransmitError>;
    /// Waits for capacity, then claims a slot holding a recycled value to write into
    async fn async_send_ref(&self) -> Result<Self::SendGuard<'_>, FeagiChannelTransmitError>;
    /// Claims a slot holding a recycled value to write into without waiting
    fn try_send_ref(&self) -> Result<Self::SendGuard<'_>, FeagiChannelTransmitError>;
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

    /// Waits for the next value. Errors with `ChannelClosed` once every transmitter is gone and the
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
    // TODO: Disabled because thingbuf 0.1.6 `Receiver::is_closed` reports closed while one transmitter
    // is still alive (it checks `tx_count <= 1`, but the count starts at 1 for the first transmitter).
    // Restore once thingbuf is fixed or patched. Receiving still reports `ChannelClosed` correctly
    // /// True if every transmitter has been dropped. Items already queued can still be received
    // fn is_closed(&self) -> bool;
}