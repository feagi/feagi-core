#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum FeagiClientConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FeagiClientConnectionStateChange {
    previous: FeagiClientConnectionState,
    now: FeagiClientConnectionState,
}

impl FeagiClientConnectionStateChange {
    pub fn new(previous: FeagiClientConnectionState, now: FeagiClientConnectionState) -> Self {
        Self { previous, now }
    }

    pub fn previous(&self) -> FeagiClientConnectionState {
        self.previous
    }

    pub fn now(&self) -> FeagiClientConnectionState {
        self.now
    }
}

/// Type alias for the client state change callback.
pub type StateChangeCallback = Box<dyn Fn(FeagiClientConnectionStateChange) + Send + Sync + 'static>;