use crate::next::network_temp::FeagiNetworkError;
use crate::next::network_temp::traits::server::feagi_server::FeagiServer;

pub trait FeagiServerRouter: FeagiServer {
    fn set_request_processing_function<F>(&self, on_callback: F) where
        F: Fn(&[u8]) -> &[u8] + Send + Sync + 'static;

    fn get_set_request_processing_function<F>(&self) -> Result<F, FeagiNetworkError> where
        F: Fn(&[u8]) -> &[u8] + Send + Sync + 'static;

    fn send_response(&self, response_data: &[u8]);

    fn received_request(&self, request_data: &[u8]) -> Result<(), FeagiNetworkError> {
        let processing_function = self.get_set_request_processing_function()?;
        let processed_data = processing_function(request_data);
        self.send_response(processed_data);
    }
}