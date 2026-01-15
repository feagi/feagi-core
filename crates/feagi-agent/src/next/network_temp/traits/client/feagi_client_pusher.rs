use crate::next::network_temp::traits::client::feagi_client::FeagiClient;

pub trait FeagiClientPusher: FeagiClient {
    fn push_data(&self, data: &[u8]);
}