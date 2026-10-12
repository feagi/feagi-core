use thiserror::Error;

#[derive(Error, Debug)]
pub enum FeagiRestServerError {
    #[error("Server failed to bind to socket!")] // TODO socket?
    FailedToBindToSocket,
    #[error("Server failed to parse data!")]
    ServerIncomingDataParseFailure,
}
