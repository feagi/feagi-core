use core::marker::PhantomData;
use core::time::Duration;
use flume::{TryRecvError, TrySendError};
use crate::data_channels::data_channel::{DataChannelPair, DataReceiver, DataTransmitter};
use crate::data_channels::data_cycler::DataCycleEndpoint;
use crate::data_channels::errors::{ChannelReceivingError, ChannelSendingError, FeagiFailChannelReceiveEtc, FeagiFailChannelReceiveTimeout, FeagiFailChannelSendEtc, FeagiFailChannelSendFull, FeagiFailChannelSendTimeout};
//region Data Channel Pair

pub struct FlumeChannelPair<T: Send>(PhantomData<T>);

impl<T: Send> DataChannelPair<T> for FlumeChannelPair<T> {
    type Transmitter = FlumeTransmitter<T>;
    type Receiver = FlumeReceiver<T>;

    fn new_pair(buffer_length: usize) -> (Self::Transmitter, Self::Receiver) {
        let (t, r) = flume::bounded(buffer_length);
        (FlumeTransmitter(t), FlumeReceiver(r))
    }
}

pub struct FlumeTransmitter<T: Send>(flume::Sender<T>);

impl<T: Send> DataTransmitter<T> for FlumeTransmitter<T> {
    fn block_send(&mut self, sending: T) -> Result<(), ChannelSendingError> {
        self.0.send(sending).map_err(
            |_|
                FeagiFailChannelSendEtc::new("Failed to block send data over channel").into()
        )
    }

    fn try_send(&mut self, sending: T) -> Result<(), ChannelSendingError> {
        self.0.try_send(sending).map_err(
            |e|
                match e {
                    TrySendError::Full(_) => {
                        FeagiFailChannelSendFull::new("Failed to try send data, channel is full!").into()
                    }
                    TrySendError::Disconnected(_) => {
                        FeagiFailChannelSendEtc::new("Failed to try send data over disconnected channel").into()
                    }
                }
        )
    }

    fn send_timeout(&mut self, sending: T, timeout: Duration) -> Result<(), ChannelSendingError> {
        self.0.send_timeout(sending, timeout).map_err(
            |e| match e {
                flume::SendTimeoutError::Timeout(_) => {
                    FeagiFailChannelSendTimeout::new("Failed to send data over channel before timeout expired").into()
                }
                flume::SendTimeoutError::Disconnected(_) => {
                    FeagiFailChannelSendEtc::new("Failed to send data over disconnected channel").into()
                }
            },
        )
    }
}

pub struct FlumeReceiver<T: Send>(flume::Receiver<T>);

impl<T: Send> DataReceiver<T> for FlumeReceiver<T> {
    fn block_receive(&mut self) -> Result<T, ChannelReceivingError> {
        self.0.recv().map_err(
            |_| FeagiFailChannelReceiveEtc::new("Failed to block receive data over channel").into(),
        )
    }

    fn try_receive(&mut self) -> Result<Option<T>, ChannelReceivingError> {
        let res = self.0.try_recv();
        match res {
            Ok(o) => {
                Ok(Some(o))
            }
            Err(e) => {
                match e {
                    TryRecvError::Empty => {
                        Ok(None)
                    }
                    TryRecvError::Disconnected => {
                        Err(FeagiFailChannelReceiveEtc::new("Failed to try receive data over disconnected channel").into())
                    }
                }
            }
        }
    }

    fn receive_timeout(&mut self, timeout: Duration) -> Result<T, ChannelReceivingError> {
        self.0.recv_timeout(timeout).map_err(
            |e| match e {
                flume::RecvTimeoutError::Timeout => {
                    FeagiFailChannelReceiveTimeout::new("Failed to receive data over channel before timeout expired").into()
                }
                flume::RecvTimeoutError::Disconnected => {
                    FeagiFailChannelReceiveEtc::new("Failed to receive data over disconnected channel").into()
                }
            },
        )
    }
}


//endregion

//region Data Cycler

/// Allows reusing a block of memory back and forth
pub struct FlumeDataCycleEndpoint<T: Send> {
    transmitter: FlumeTransmitter<T>,
    receiver: FlumeReceiver<T>,
}

impl<T: Send> DataCycleEndpoint<T> for FlumeDataCycleEndpoint<T> {
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

    fn block_receive(&mut self) -> Result<T, ChannelReceivingError> {
        self.receiver.block_receive()
    }

    fn try_receive(&mut self) -> Result<Option<T>, ChannelReceivingError> {
        self.receiver.try_receive()
    }

    fn receive_timeout(&mut self, timeout: Duration) -> Result<T, ChannelReceivingError> {
        self.receiver.receive_timeout(timeout)
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

//endregion