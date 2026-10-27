use thiserror::Error;

/// Every notifier of the signal was dropped, and no value is left to take
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("Feagi signal is closed, every notifier was dropped!")]
pub struct FeagiSignalClosed;
