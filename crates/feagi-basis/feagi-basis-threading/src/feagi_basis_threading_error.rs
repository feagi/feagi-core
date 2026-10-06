use crate::thread_messaging::{ChannelError, ThreadMessagingError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FeagiBasisThreadingError {
    #[error(transparent)]
    ChannelError(#[from] ChannelError),
    #[error(transparent)]
    ThreadMessagingError(#[from] ThreadMessagingError),
}
