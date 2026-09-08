use core::time::Duration;
use std::marker::PhantomData;
use std::sync::mpsc::{self, TryRecvError, TrySendError};

use crate::data_messaging::data_channel::{DataChannelPair, DataReceiver, DataTransmitter};
use crate::data_messaging::errors::{
    ChannelReceivingError, ChannelSendingError, FeagiFailChannelReceiveEtc,
    FeagiFailChannelReceiveTimeout, FeagiFailChannelSendEtc, FeagiFailChannelSendFull,
    FeagiFailChannelSendTimeout,
};

pub struct MpscChannelPair<T: Send>(PhantomData<T>);

impl<T: Send> MpscChannelPair<T> {
    pub fn new_pair(buffer_length: usize, polling_interval: Duration) -> (<MpscChannelPair<T> as DataChannelPair<T>>::Transmitter, <MpscChannelPair<T> as DataChannelPair<T>>::Receiver) {
        let (transmitter, receiver) = mpsc::sync_channel(buffer_length);
        (MpscTransmitter(transmitter, polling_interval), MpscReceiver(receiver))
    }
}

impl<T: Send> DataChannelPair<T> for MpscChannelPair<T> {
    type Transmitter = MpscTransmitter<T>;
    type Receiver = MpscReceiver<T>;


}

pub struct MpscTransmitter<T: Send>(mpsc::SyncSender<T>, Duration);

impl<T: Send> MpscTransmitter<T> {
    /// Tries to send without converting errors to `ChannelSendingError`.
    pub fn try_send_raw(&self, sending: T) -> Result<(), TrySendError<T>> {
        self.0.try_send(sending)
    }
}

impl<T: Send> DataTransmitter<T> for MpscTransmitter<T> {
    fn block_send(&mut self, sending: T) -> Result<(), ChannelSendingError> {
        self.0.send(sending).map_err(|_| {
            FeagiFailChannelSendEtc::new("Failed to block send data over channel").into()
        })
    }

    fn try_send(&mut self, sending: T) -> Result<(), ChannelSendingError> {
        self.0.try_send(sending).map_err(|error| match error {
            TrySendError::Full(_) => {
                FeagiFailChannelSendFull::new("Failed to try send data, channel is full!").into()
            }
            TrySendError::Disconnected(_) => {
                FeagiFailChannelSendEtc::new("Failed to try send data over disconnected channel")
                    .into()
            }
        })
    }

    fn send_timeout(&mut self, sending: T, timeout: Duration) -> Result<(), ChannelSendingError> {
        let deadline = std::time::Instant::now() + timeout;
        let mut item = sending;

        loop {
            match self.0.try_send(item) {
                Ok(()) => return Ok(()),
                Err(TrySendError::Disconnected(_)) => {
                    return Err(
                        FeagiFailChannelSendEtc::new(
                            "Failed to send data over disconnected channel",
                        )
                        .into(),
                    );
                }
                Err(TrySendError::Full(returned)) => {
                    item = returned;
                    if std::time::Instant::now() >= deadline {
                        return Err(
                            FeagiFailChannelSendTimeout::new(
                                "Failed to send data over channel before timeout expired",
                            )
                            .into(),
                        );
                    }

                    std::thread::sleep(self.1); // TODO swap to a timer that polls using while when time gets closer
                }
            }
        }
    }
}

pub struct MpscReceiver<T: Send>(mpsc::Receiver<T>);

impl<T: Send> MpscReceiver<T> {
    /// Tries to receive without converting errors to `ChannelReceivingError`.
    pub fn try_recv_raw(&self) -> Result<T, TryRecvError> {
        self.0.try_recv()
    }
}

impl<T: Send> DataReceiver<T> for MpscReceiver<T> {
    fn block_receive(&mut self) -> Result<T, ChannelReceivingError> {
        self.0.recv().map_err(|_| {
            FeagiFailChannelReceiveEtc::new("Failed to block receive data over channel").into()
        })
    }

    fn try_receive(&mut self) -> Result<Option<T>, ChannelReceivingError> {
        match self.0.try_recv() {
            Ok(item) => Ok(Some(item)),
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => Err(
                FeagiFailChannelReceiveEtc::new(
                    "Failed to try receive data over disconnected channel",
                )
                .into(),
            ),
        }
    }

    fn receive_timeout(&mut self, timeout: Duration) -> Result<T, ChannelReceivingError> {
        self.0.recv_timeout(timeout).map_err(|error| match error {
            mpsc::RecvTimeoutError::Timeout => {
                FeagiFailChannelReceiveTimeout::new(
                    "Failed to receive data over channel before timeout expired",
                )
                .into()
            }
            mpsc::RecvTimeoutError::Disconnected => {
                FeagiFailChannelReceiveEtc::new(
                    "Failed to receive data over disconnected channel",
                )
                .into()
            }
        })
    }
}
