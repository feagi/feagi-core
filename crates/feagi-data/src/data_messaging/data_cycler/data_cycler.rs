//! A trait for a pair of structs that can be used to send data over a thread boundary, but 
//! rotate the allocated memory to do so amongst themselves, avoiding any dynamic allocation

use core::time::Duration;
use crate::data_messaging::errors::{ChannelReceivingError, ChannelSendingError};

pub trait DataCycleEndpoint<T: Send>: Send + Sized {
    /// Waits to receive data over the cycle, blocking the thread until it does (or erroring)
    fn block_receive(&mut self) -> Result<T, ChannelReceivingError>;

    /// Tries to receive data from the cycle, returning None if nothing is available
    fn try_receive(&mut self) -> Result<Option<T>, ChannelReceivingError>;

    /// Waits to receive data over the cycle, blocking the thread until a timeout (or erroring)
    fn receive_timeout(&mut self, timeout: Duration) -> Result<T, ChannelReceivingError>;

    /// Enqueue data over the cycle, blocking the thread until it does (or erroring)
    fn block_enqueue(&mut self, sending: T) -> Result<(), ChannelSendingError>;

    /// Tries to enqueue data over the cycle, erroring immediately if it cannot
    fn try_enqueue(&mut self, sending: T) -> Result<(), ChannelSendingError>;

    /// Enqueue data over the cycle, blocking the thread until it does or a timeout expires (or erroring)
    fn enqueue_timeout(&mut self, sending: T, timeout: Duration) -> Result<(), ChannelSendingError>;

    /// Return data over the channel, blocking the thread until it does (or erroring)
    fn block_return(&mut self, returning: T) -> Result<(), ChannelSendingError>;

    /// Tries to return data over the channel, erroring immediately if it cannot
    fn try_return(&mut self, returning: T) -> Result<(), ChannelSendingError>;

    /// Return data over the channel, blocking the thread until it does until a timeout (or erroring)
    fn return_timeout(&mut self, returning: T, timeout: Duration) -> Result<(), ChannelSendingError>;
}


