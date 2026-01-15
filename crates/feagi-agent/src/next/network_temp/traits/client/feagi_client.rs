use crate::next::network_temp::error::FeagiNetworkError;
use crate::next::network_temp::state_enums::FeagiConnectionState;

pub trait FeagiClient {
    fn connect(&self, host: String);
    fn disconnect(&self) -> Result<(), FeagiNetworkError>;
    fn get_current_connection_state(&self) -> FeagiConnectionState;
    fn register_connection_state_changes<F>(&self, on_state_change: F) where
        F: Fn((FeagiConnectionState, FeagiConnectionState)) + Send + Sync + 'static;
}
