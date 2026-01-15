use crate::next::network_temp::traits::server::feagi_server::FeagiServer;

pub trait FeagiServerPublisher: FeagiServer {
    fn publish(&self, data_to_send: &[u8]);
}