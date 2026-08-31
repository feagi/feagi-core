use std::time::Duration;
use crate::data_channels::errors::{ChannelReceivingError, ChannelSendingError};

/// Allows creations of `DataTransmitter` and `DataReceiver` easily
pub trait DataChannelPair<T: Send> {
    type Transmitter;
    type Receiver;
    fn new_pair(buffer_length: usize) -> (Self::Transmitter, Self::Receiver);
}

/// Generic trait for a struct that can send data to a paired `DataReceiver`
pub trait DataTransmitter<T: Send> {
    /// Send data over the channel, blocking the thread until it does (or erroring)
    fn block_send(&mut self, sending: T) -> Result<(), ChannelSendingError>;

    /// Tries to send data over the channel, erroring immediately if it cannot
    fn try_send(&mut self, sending: T) -> Result<(), ChannelSendingError>;

    /// Send data over the channel, blocking the thread until it does until a timeout (or erroring)
    fn send_timeout(&mut self, sending: T, timeout: Duration) -> Result<(), ChannelSendingError>;
}

/// Generic trait for a struct that can get data from a paired `DataTransmitter`
pub trait DataReceiver<T: Send> {
    /// Waits to receive data over the channel, blocking the thread until it does (or erroring)
    fn block_receive(&mut self) -> Result<T, ChannelReceivingError>;

    /// Tries to receive data from the channel, returning None if nothing is available
    fn try_receive(&mut self) -> Result<Option<T>, ChannelReceivingError>;

    /// Waits to receive data over the channel, blocking the thread until a timeout (or erroring)
    fn receive_timeout(&mut self, timeout: Duration) -> Result<T, ChannelReceivingError>;
}