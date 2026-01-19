use crate::next::FeagiNetworkError;
use crate::next::state_enums::FeagiClientConnectionState;

pub trait FeagiClient {
    fn connect(&self, host: String);
    fn disconnect(&self) -> Result<(), FeagiNetworkError>;
    fn get_current_connection_state(&self) -> FeagiClientConnectionState;
    fn register_connection_state_changes<F>(&self, on_state_change: F) where
        F: Fn((FeagiClientConnectionState, FeagiClientConnectionState)) + Send + Sync + 'static;
}
