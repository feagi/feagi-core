use crate::next::network_temp::traits::server::feagi_server::FeagiServer;

pub trait FeagiServerPuller: FeagiServer {
    fn set_callback_for_data_received<F>(&self, data_to_send: &[u8]);
}