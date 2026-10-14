use thiserror::Error;

/// The sender failed to send data. The data that was tried to be sent is dropped
#[derive(Debug, Error)]
pub enum FeagiChannelSendError {
    #[error("Feagi channel is closed, receiver likely terminated unexpectedly!")]
    ChannelClosed,
    #[error("Feagi channel is full, receiver is likely unresponsive!")]
    ChannelFull,
    #[error("Feagi channel is full even after awaiting, receiver is likely unresponsive!")]
    ChannelFullAfterExceedingTimeout,
}

impl<T> From<thingbuf::mpsc::errors::TrySendError<T>> for FeagiChannelSendError {
    fn from(value: thingbuf::mpsc::errors::TrySendError<T>) -> Self {
        match value {
            thingbuf::mpsc::errors::TrySendError::Full(_) => {FeagiChannelSendError::ChannelFull}
            thingbuf::mpsc::errors::TrySendError::Closed(_) => {FeagiChannelSendError::ChannelClosed}
            _ => {unreachable!("Invalid send channel error!")}
        }
    }
}

impl<T> From<thingbuf::mpsc::errors::Closed<T>> for FeagiChannelSendError {
    fn from(_value: thingbuf::mpsc::errors::Closed<T>) -> Self {
        FeagiChannelSendError::ChannelClosed
    }
}

impl<T> From<thingbuf::mpsc::errors::SendTimeoutError<T>> for FeagiChannelSendError {
    fn from(value: thingbuf::mpsc::errors::SendTimeoutError<T>) -> Self {
        match value {
            thingbuf::mpsc::errors::SendTimeoutError::Timeout(_) => {FeagiChannelSendError::ChannelFullAfterExceedingTimeout}
            thingbuf::mpsc::errors::SendTimeoutError::Closed(_) => {FeagiChannelSendError::ChannelClosed}
            _ => {unreachable!("Invalid send channel error!")}
        }
    }
}


/// The receiver was unable to receive data. This may either be expected (no data was sent) or
/// the channel was closed
#[derive(Debug, Error)]
pub enum FeagiChannelReceiveError {
    #[error("Feagi channel is empty. This is not an error")]
    ChannelEmpty,
    #[error("Feagi channel is closed, sender likely terminated unexpectedly!")]
    ChannelClosed,
    #[error("Feagi channel is empty even after awaiting!")]
    ChannelEmptyAfterExceedingTimeout,
}

impl From<thingbuf::mpsc::errors::TryRecvError> for FeagiChannelReceiveError {
    fn from(value: thingbuf::mpsc::errors::TryRecvError) -> Self {
        match value {
            thingbuf::mpsc::errors::TryRecvError::Empty => {FeagiChannelReceiveError::ChannelEmpty}
            thingbuf::mpsc::errors::TryRecvError::Closed => {FeagiChannelReceiveError::ChannelClosed}
            _ => {unreachable!("Invalid receive channel error!")}
        }
    }
}

impl From<thingbuf::mpsc::errors::RecvTimeoutError> for FeagiChannelReceiveError {
    fn from(value: thingbuf::mpsc::errors::RecvTimeoutError) -> Self {
        match value {
            thingbuf::mpsc::errors::RecvTimeoutError::Timeout => {FeagiChannelReceiveError::ChannelEmptyAfterExceedingTimeout}
            thingbuf::mpsc::errors::RecvTimeoutError::Closed => {FeagiChannelReceiveError::ChannelClosed}
            _ => {unreachable!("Invalid receive channel error!")}
        }
    }
}


/// A request made over a feagi request / response channel did not produce a response
#[derive(Debug, Error)]
pub enum FeagiRequestError {
    #[error("Feagi request could not be sent: {0}")]
    RequestNotSent(#[from] FeagiChannelSendError),
    #[error("Feagi reply could not be received: {0}")]
    ReplyNotReceived(#[from] FeagiChannelReceiveError),
    #[error("Feagi responder dropped the request without responding!")]
    ResponderDroppedRequest,
}
