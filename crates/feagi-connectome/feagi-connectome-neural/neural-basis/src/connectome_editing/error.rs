

#[derive(Debug, thiserror::Error)]
pub enum ConnectomeEditError {
    #[error("Connectome Editing had an unknown failure. Corruption assumed")]
    UnknownFailure
}