#[cfg(feature = "transport_rest")]
use crate::transport::rest::server::error::FeagiRestServerError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FeagiInteropError {
    #[error("Failed to find to socket")]
    FailedToBindSocket,
    #[cfg(feature = "transport_rest")]
    #[error(transparent)]
    RestServerError(#[from] FeagiRestServerError),
}
