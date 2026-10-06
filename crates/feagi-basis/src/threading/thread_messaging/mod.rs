//! Structures for sending data between threads, with implementations using different backends
//! for different use cases.

pub use crate::channel::channel_error::{ChannelError, ChannelReceivingError, ChannelSendingError, ThreadMessagingError};
// pub mod data_channel_pair;  TODO migrate from flume
//pub mod data_cycler; TODO migrate from flume
pub mod multi_request_channel;
