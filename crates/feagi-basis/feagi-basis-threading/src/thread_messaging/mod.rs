//! Structures for sending data between threads, with implementations using different backends
//! for different use cases.

mod errors;

pub use errors::{ChannelError, ChannelReceivingError, ChannelSendingError, ThreadMessagingError};
// pub mod data_channel_pair;  TODO migrate from flume
//pub mod data_cycler; TODO migrate from flume
pub mod multi_request_channel;
