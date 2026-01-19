use crate::next::traits::server::FeagiServer;

pub trait FeagiServerPuller: FeagiServer {
    // Technically no functions, however the new function should take in a
    // "Fn(&[u8]) -> &[u8] + Send + Sync + 'static" to call when data is received
}