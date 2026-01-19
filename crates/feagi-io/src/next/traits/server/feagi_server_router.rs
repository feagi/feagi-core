use crate::next::FeagiNetworkError;
use crate::next::traits::server::FeagiServer;

pub trait FeagiServerRouter: FeagiServer {

    // New function needs to take in a request processing function of type
    // "Fn(&[u8], &mut [u8])  Send + Sync + 'static"

    /// INTERNAL ONLY
    fn _received_request(&self, request_data: &[u8]) -> Result<(), FeagiNetworkError>;
    // On data being received, run it through the internal function, get the output data, then
    // call _send_response with it

    /// INTERNAL ONLY
    fn _send_response(&self);
}