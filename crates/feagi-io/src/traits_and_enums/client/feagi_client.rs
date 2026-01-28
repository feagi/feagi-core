use std::future::Future;

use crate::FeagiNetworkError;
use crate::traits_and_enums::client::client_shared::FeagiClientConnectionState;

pub trait FeagiClient: Send {
    fn connect(&mut self, host: &str) -> impl Future<Output = Result<(), FeagiNetworkError>>;
    fn disconnect(&mut self) -> impl Future<Output = Result<(), FeagiNetworkError>>;
    fn get_current_connection_state(&self) -> FeagiClientConnectionState;
}
