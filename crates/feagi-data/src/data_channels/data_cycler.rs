use std::time::Duration;
use crate::data_channels::errors::{ChannelReceivingError, ChannelSendingError};

pub trait DataCycleEndpoint<T: Send>: Send + Sized {
    fn new_data_cycle_endpoint_pair(buffer_length: usize) -> (Self, Self);

    /// Waits to receive data over the cycle, blocking the thread until it does (or erroring)
    fn block_receive(&mut self) -> Result<T, ChannelReceivingError>;

    /// Tries to receive data from the cycle, returning None if nothing is available
    fn try_receive(&mut self) -> Result<Option<T>, ChannelReceivingError>;

    /// Waits to receive data over the cycle, blocking the thread until a timeout (or erroring)
    fn receive_timeout(&mut self, timeout: Duration) -> Result<T, ChannelReceivingError>;

    /// Return data over the channel, blocking the thread until it does (or erroring)
    fn block_return(&mut self, returning: T) -> Result<(), ChannelSendingError>;

    /// Tries to return data over the channel, erroring immediately if it cannot
    fn try_return(&mut self, returning: T) -> Result<(), ChannelSendingError>;

    /// Return data over the channel, blocking the thread until it does until a timeout (or erroring)
    fn return_timeout(&mut self, returning: T, timeout: Duration) -> Result<(), ChannelSendingError>;
}


