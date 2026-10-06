use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChannelError {
    #[error("Failed to send because the channel is full")]
    SendFull,
    #[error("Failed to send before the timeout expired")]
    SendTimeout,
    #[error("Failed to send over the channel")]
    SendEtc,
    #[error("The channel has been closed")]
    SendClosed,
    #[error("Failed to receive before the timeout expired")]
    ReceiveTimeout,
    #[error("Failed to receive over the channel")]
    ReceiveEtc,
    #[error("The pool is full")]
    PoolFull,
    #[error("The pool is empty")]
    PoolEmpty,
}

impl<T> From<thingbuf::mpsc::errors::Closed<T>> for ChannelError {
    fn from(_: thingbuf::mpsc::errors::Closed<T>) -> Self {
        ChannelError::SendClosed
    }
}

#[derive(Error, Debug)]
pub enum ChannelSendingError {
    #[error("Failed to send over the channel")]
    SendFailed,
    #[error("Failed to send because the channel is full")]
    SendChannelFull,
    #[error("Failed to send before the timeout expired")]
    SendTimeout,
}

#[derive(Error, Debug)]
pub enum ChannelReceivingError {
    #[error("Failed to receive over the channel")]
    ReceiveFailed,
    #[error("Failed to receive before the timeout expired")]
    ReceiveTimeout,
}

#[derive(Error, Debug)]
pub enum ThreadMessagingError {
    #[error("Some other thread messaging error occurred")]
    Etc,
    #[error(transparent)]
    SendingError(#[from] ChannelSendingError),
    #[error(transparent)]
    ReceivingError(#[from] ChannelReceivingError),
}
