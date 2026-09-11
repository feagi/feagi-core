use core::time::Duration;

use crate::thread_messaging::data_channel::{DataChannelPair, DataReceiver, DataTransmitter};
use crate::thread_messaging::data_channel::implementations::mpsc::{
    MpscChannelPair, MpscReceiver, MpscTransmitter,
};
use crate::thread_messaging::data_cycler::DataCycleEndpoint;
use crate::thread_messaging::errors::{ChannelReceivingError, ChannelSendingError};

/// Allows reusing a block of memory back and forth using `std::sync::mpsc`.
pub struct MpscDataCycleEndpoint<T: Send> {
    transmitter: MpscTransmitter<T>,
    receiver: MpscReceiver<T>,
}

impl<T: Send> MpscDataCycleEndpoint<T> {
    /// Creates a pair of endpoints with independent buffer sizes for each direction.
    pub fn new_data_cycle_endpoint_pair(
        buffer_length: usize,
        polling_interval: Duration
        
    ) -> (Self, Self) {
        let (a_to_b_transmitter, a_to_b_receiver) = MpscChannelPair::new_pair(buffer_length, polling_interval);
        let (b_to_a_transmitter, b_to_a_receiver) = MpscChannelPair::new_pair(buffer_length, polling_interval);

        (
            Self {
                transmitter: a_to_b_transmitter,
                receiver: b_to_a_receiver,
            },
            Self {
                transmitter: b_to_a_transmitter,
                receiver: a_to_b_receiver,
            },
        )
    }
}

impl<T: Send> DataCycleEndpoint<T> for MpscDataCycleEndpoint<T> {

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
