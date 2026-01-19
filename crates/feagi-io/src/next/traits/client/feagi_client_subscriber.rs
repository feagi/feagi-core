use crate::next::traits::client::feagi_client::FeagiClient;

pub trait FeagiClientSubscriber: FeagiClient {
    // No functions, but new must take in a "F: Fn(&[u8]) -> &[u8] + Send + Sync + 'static"
}