use crate::FeagiNetworkError;
use crate::traits_and_enums::client::client_shared::FeagiClientConnectionState;

pub trait FeagiClient: Send {
    fn connect(&mut self, host: &str) -> Result<(), FeagiNetworkError>;
    fn disconnect(&mut self) -> Result<(), FeagiNetworkError>;
    fn get_current_connection_state(&self) -> FeagiClientConnectionState;
}
