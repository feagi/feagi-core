use crate::next::traits_and_enums::client::feagi_client::FeagiClient;

pub trait FeagiClientSubscriber: FeagiClient {
    // No functions, but new must take in a "F: Fn(&[u8])"
}