use crate::parallelism::feagi_channel::base_channels::base_errors::{FeagiChannelReceiveError, FeagiChannelTransmitError};
use thiserror::Error;

/// A request made over a feagi request / response channel did not produce a response
#[derive(Debug, Error)]
pub enum FeagiRequestResponseError {
    #[error("Feagi request could not be sent: {0}")]
    RequestNotSent(#[from] FeagiChannelTransmitError),
    #[error("Feagi reply could not be received: {0}")]
    ReplyNotReceived(#[from] FeagiChannelReceiveError),
    #[error("Feagi responder dropped the request without responding!")]
    ResponderDroppedRequest,
    #[error("Feagi responder replied with a response that does not match the request!")]
    UnexpectedResponse,
}
