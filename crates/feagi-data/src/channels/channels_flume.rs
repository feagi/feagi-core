use std::time::Duration;
use flume::{TryRecvError, TrySendError};
use crate::channels::channels::{ChannelReceivingError, ChannelSendingError, FeagiFailChannelReceiveEtc, FeagiFailChannelReceiveTimeout, FeagiFailChannelSendEtc, FeagiFailChannelSendFull, FeagiFailChannelSendTimeout, ChannelPair};

// TODO fix in out naming conventions

pub type InnerFlumeChannelPair<ComingFromOutside: Send, GoingOutside: Send> =  FlumeChannelPair<ComingFromOutside, GoingOutside>;
pub type OuterFlumeChannelPair<GoingInside: Send, ComingFromInside: Send> =  FlumeChannelPair<GoingInside, ComingFromInside>;

pub struct FlumeChannelPair<ToIn: Send, ToOut: Send> {
    a_sender: flume::Sender<ToIn>,
    b_receiver: flume::Receiver<ToOut>,
}

impl<ToIn: Send, ToOut: Send> ChannelPair<ToIn, ToOut> for FlumeChannelPair<ToIn, ToOut> {
    type AChannelPair = InnerFlumeChannelPair<ToOut, ToIn>;
    type BChannelPair = OuterFlumeChannelPair<ToIn, ToOut>;

    fn new_pairs(going_in_length: usize, going_out_length: usize) -> (OuterFlumeChannelPair<ToIn, ToOut>, InnerFlumeChannelPair<ToOut, ToIn>) {
        let (to_in_tx, to_in_rx) = flume::bounded(going_in_length);
        let (to_out_tx, to_out_rx) = flume::bounded(going_out_length);
        (
            FlumeChannelPair {
                a_sender: to_in_tx,
                b_receiver: to_out_rx
            },
            FlumeChannelPair {
                a_sender: to_out_tx,
                b_receiver: to_in_rx
            },
        )
    }
    fn block_send(&mut self, to_in: ToIn) -> Result<(), ChannelSendingError> {
        self.a_sender.send(to_in).map_err(
            |_|
                FeagiFailChannelSendEtc::new("Failed to block send data over channel").into()
        )
    }
    fn try_send(&mut self, to_in: ToIn) -> Result<(), ChannelSendingError> {
        self.a_sender.try_send(to_in).map_err(
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
    fn send_timeout(&mut self, to_in: ToIn, timeout: Duration) -> Result<(), ChannelSendingError> {
        self.a_sender.send_timeout(to_in, timeout).map_err(
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
    fn block_receive(&mut self) -> Result<ToOut, ChannelReceivingError> {
        self.b_receiver.recv().map_err(
            |_| FeagiFailChannelReceiveEtc::new("Failed to block receive data over channel").into(),
        )
    }
    fn try_receive(&mut self) -> Result<Option<ToOut>, ChannelReceivingError> {
        let res = self.b_receiver.try_recv();
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
    fn receive_timeout(&mut self, timeout: Duration) -> Result<ToOut, ChannelReceivingError> {
        self.b_receiver.recv_timeout(timeout).map_err(
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



