use std::future::Future;
use crate::FeagiNetworkError;
use crate::traits_and_enums::client::FeagiClient;

pub trait FeagiClientSubscriber: FeagiClient {
    /// get incoming data
    fn get_subscribed_data(&mut self) -> impl Future<Output = Result<&[u8], FeagiNetworkError>>;
}
