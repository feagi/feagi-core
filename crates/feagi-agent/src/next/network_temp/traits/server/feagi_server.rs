use crate::next::network_temp::error::FeagiNetworkError;

pub trait FeagiServer {
    fn start(&self) -> Result<(), FeagiNetworkError>;
    fn stop(&self) -> Result<(), FeagiNetworkError>;
    fn is_running(&self) -> bool;
}