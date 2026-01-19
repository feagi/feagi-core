use crate::next::FeagiNetworkError;
use crate::next::traits::server::FeagiServer;

pub trait FeagiServerPublisher: FeagiServer {
    fn publish(&mut self, buffered_data_to_send: &[u8]) -> Result<(), FeagiNetworkError>;
}