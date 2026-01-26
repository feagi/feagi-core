use crate::traits_and_enums::client::FeagiClient;

pub trait FeagiClientSubscriber: FeagiClient {
    // No functions, but new must take in a "F: Fn(&[u8])"
}
