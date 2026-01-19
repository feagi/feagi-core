use crate::next::traits::client::feagi_client::FeagiClient;

pub trait FeagiClientRequester: FeagiClient {
    fn send_request_and_process_response<F>(&self, request: &[u8], on_response_received: F) where
        F: Fn(&[u8]) -> &[u8] + Send + Sync + 'static;
}