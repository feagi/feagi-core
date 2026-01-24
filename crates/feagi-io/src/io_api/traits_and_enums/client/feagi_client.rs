use crate::io_api::traits_and_enums::client::client_shared::FeagiClientConnectionState;
use crate::io_api::FeagiNetworkError;

pub trait FeagiClient: Send {
    fn connect(&mut self, host: &str) -> Result<(), FeagiNetworkError>;
    fn disconnect(&mut self) -> Result<(), FeagiNetworkError>;
    fn get_current_connection_state(&self) -> FeagiClientConnectionState;
}
