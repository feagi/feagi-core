use crate::traits_and_enums::client::FeagiClient;

pub trait FeagiClientPusher: FeagiClient {
    fn push_data(&self, data: &[u8]);
}
