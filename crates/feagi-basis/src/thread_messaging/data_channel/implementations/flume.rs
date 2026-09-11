use core::marker::PhantomData;
use core::time::Duration;
use flume::{TryRecvError, TrySendError};
use crate::thread_messaging::data_channel::{DataChannelPair, DataReceiver, DataTransmitter};
use crate::thread_messaging::errors::{ChannelReceivingError, ChannelSendingError, FeagiFailChannelReceiveEtc, FeagiFailChannelReceiveTimeout, FeagiFailChannelSendEtc, FeagiFailChannelSendFull, FeagiFailChannelSendTimeout};

pub struct FlumeChannelPair<T: Send>(PhantomData<T>);

impl<T: Send> FlumeChannelPair<T> {
    pub fn new_pair(buffer_length: usize) -> (<FlumeChannelPair<T> as DataChannelPair<T>>::Transmitter, <FlumeChannelPair<T> as DataChannelPair<T>>::Receiver) {
        let (t, r) = flume::bounded(buffer_length);
        (FlumeTransmitter(t), FlumeReceiver(r))
    }
}

impl<T: Send> DataChannelPair<T> for FlumeChannelPair<T> {
    type Transmitter = FlumeTransmitter<T>;
    type Receiver = FlumeReceiver<T>;
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