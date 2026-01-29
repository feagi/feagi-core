use std::future::Future;
use crate::FeagiNetworkError;
use crate::traits_and_enums::client::FeagiClient;

pub trait FeagiClientPusher: FeagiClient {
    fn push_data(&mut self, data: &[u8]) -> impl Future<Output = Result<(), FeagiNetworkError>>;
}
