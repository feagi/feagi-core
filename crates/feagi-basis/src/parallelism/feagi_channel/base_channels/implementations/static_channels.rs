use crate::parallelism::feagi_channel::base_channels::base_channels::{FeagiChannelReceiver, FeagiChannelTransmitter};
use crate::parallelism::feagi_channel::base_channels::base_errors::{FeagiChannelReceiveError, FeagiChannelTransmitError};
use crate::parallelism::feagi_channel::recycling::{Recyclable, RecyclableRecyclePolicy};

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

    /// Splits the channel into its transmitter and receiver
    /// A static channel can only be split once.
    pub fn create_channels(&'static self) -> (thingbuf::mpsc::StaticSender<T, RecyclableRecyclePolicy<T>>, thingbuf::mpsc::StaticReceiver<T, RecyclableRecyclePolicy<T>>) {
        self.0.split()
    }
}

impl<T: Recyclable + 'static, const CAPACITY: usize> FeagiChannelBuilder<T> for FeagiChannelStaticBuilder<T, CAPACITY> {
    type Transmitter = thingbuf::mpsc::StaticSender<T, RecyclableRecyclePolicy<T>>;
    type Receiver = thingbuf::mpsc::StaticReceiver<T, RecyclableRecyclePolicy<T>>;
}
*/

impl<T: Recyclable + 'static> FeagiChannelTransmitter<T> for thingbuf::mpsc::StaticSender<T, RecyclableRecyclePolicy<T>> {
    type SendGuard<'a>
        = thingbuf::mpsc::SendRef<'a, T>
    where
        Self: 'a;

    async fn async_send(&self, value: T) -> Result<(), FeagiChannelTransmitError> {
        thingbuf::mpsc::StaticSender::send(self, value).await.map_err(|e| e.into())
    }

    fn try_send(&self, value: T) -> Result<(), FeagiChannelTransmitError> {
        thingbuf::mpsc::StaticSender::try_send(self, value).map_err(|e| e.into())
    }

    async fn async_send_ref(&self) -> Result<Self::SendGuard<'_>, FeagiChannelTransmitError> {
        thingbuf::mpsc::StaticSender::send_ref(self).await.map_err(|e| e.into())
    }

    fn try_send_ref(&self) -> Result<Self::SendGuard<'_>, FeagiChannelTransmitError> {
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
        thingbuf::mpsc::StaticReceiver::recv(self)
            .await
            .ok_or(FeagiChannelReceiveError::ChannelClosed)
    }

    fn try_recv(&self) -> Result<T, FeagiChannelReceiveError> {
        thingbuf::mpsc::StaticReceiver::try_recv(self).map_err(|e| e.into())
    }

    async fn async_recv_ref(&self) -> Result<Self::RecvGuard<'_>, FeagiChannelReceiveError> {
        thingbuf::mpsc::StaticReceiver::recv_ref(self)
            .await
            .ok_or(FeagiChannelReceiveError::ChannelClosed)
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
