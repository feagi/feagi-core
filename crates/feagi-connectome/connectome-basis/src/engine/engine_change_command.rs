// TODO feature gate alloc related things

/// Some other error occurred not obviously correlated to the command (possible backend hardware
/// failure)
#[derive(Debug, thiserror::Error)]
pub enum EngineChangeCommandOtherError {
    #[error("Unknown Error")]
    UnknownError
}
