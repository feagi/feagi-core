use crate::next::traits::server::FeagiServer;

pub trait FeagiServerPublisher: FeagiServer {
    fn publish(&mut self, buffered_data_to_send: &[u8]);
}