use std::time::Duration;
use crate::data_messaging::data_channel::{DataChannelPair, DataReceiver, DataTransmitter};
use crate::data_messaging::data_channel::implementations::flume::{FlumeChannelPair, FlumeReceiver, FlumeTransmitter};
use crate::data_messaging::data_cycler::DataCycleEndpoint;
use crate::data_messaging::errors::{ChannelReceivingError, ChannelSendingError};

/// Allows reusing a block of memory back and forth
pub struct FlumeDataCycleEndpoint<T: Send> {
    transmitter: FlumeTransmitter<T>,
    receiver: FlumeReceiver<T>,
}

impl<T: Send> FlumeDataCycleEndpoint<T> {
    fn new_data_cycle_endpoint_pair(buffer_length: usize) -> (Self, Self) {
        let (a, b) = FlumeChannelPair::new_pair(buffer_length);
        let (c, d) = FlumeChannelPair::new_pair(buffer_length);
        (
            Self {
                transmitter: a,
                receiver: d
            },
            Self {
                transmitter: c,
                receiver: b
            }
        )
    }
}

impl<T: Send> DataCycleEndpoint<T> for FlumeDataCycleEndpoint<T> {
    fn block_receive(&mut self) -> Result<T, ChannelReceivingError> {
        self.receiver.block_receive()
    }

    fn try_receive(&mut self) -> Result<Option<T>, ChannelReceivingError> {
        self.receiver.try_receive()
    }

    fn receive_timeout(&mut self, timeout: Duration) -> Result<T, ChannelReceivingError> {
        self.receiver.receive_timeout(timeout)
    }

    fn block_enqueue(&mut self, sending: T) -> Result<(), ChannelSendingError> {
        self.transmitter.block_send(sending)
    }

    fn try_enqueue(&mut self, sending: T) -> Result<(), ChannelSendingError> {
        self.transmitter.try_send(sending)
    }

    fn enqueue_timeout(&mut self, sending: T, timeout: Duration) -> Result<(), ChannelSendingError> {
        self.transmitter.send_timeout(sending, timeout)
    }

    fn block_return(&mut self, returning: T) -> Result<(), ChannelSendingError> {
        self.transmitter.block_send(returning)
    }

    fn try_return(&mut self, returning: T) -> Result<(), ChannelSendingError> {
        self.transmitter.try_send(returning)
    }

    fn return_timeout(&mut self, returning: T, timeout: Duration) -> Result<(), ChannelSendingError> {
        self.transmitter.send_timeout(returning, timeout)
    }
}

