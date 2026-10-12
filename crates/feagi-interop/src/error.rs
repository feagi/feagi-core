use thiserror::Error;


#[derive(Error, Debug)]
pub enum FeagiInteropError {
    #[error("Failed to find to socket")]
    FailedToBindSocket,
    //#[error(transparent)]
    //RestServerError(#[from] FeagiRestServerError),
}