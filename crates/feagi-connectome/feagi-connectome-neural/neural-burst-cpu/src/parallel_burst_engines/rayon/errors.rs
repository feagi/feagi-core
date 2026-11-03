
#[derive(Debug, thiserror::Error)]
pub enum BurstEngineCPURayonInitError {
    #[error("Unknown Error init")]
    UnknownError
}