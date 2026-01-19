use crate::next::FeagiNetworkError;
use crate::next::state_enums::FeagiServerBindState;

pub trait FeagiServer {
    fn start(&mut self) -> Result<(), FeagiNetworkError>;
    fn stop(&mut self) -> Result<(), FeagiNetworkError>;
    fn get_current_state(&self) -> FeagiServerBindState;
}