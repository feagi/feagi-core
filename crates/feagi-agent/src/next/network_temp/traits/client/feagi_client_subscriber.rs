use crate::next::network_temp::traits::client::feagi_client::FeagiClient;

pub trait FeagiClientSubscriber: FeagiClient {
    fn set_callback_for_data_received<F>(&self, on_data_received: F) where
        F: Fn(&[u8]) -> &[u8] + Send + Sync + 'static;
}