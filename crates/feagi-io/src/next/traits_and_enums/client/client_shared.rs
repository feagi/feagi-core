#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum FeagiClientConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
}