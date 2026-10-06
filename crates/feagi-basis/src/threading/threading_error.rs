use crate::threading::thread_messaging::{ChannelError, ThreadMessagingError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ThreadingError {
    #[error(transparent)]
    ChannelError(#[from] ChannelError),
    #[error(transparent)]
    ThreadMessagingError(#[from] ThreadMessagingError),
}
