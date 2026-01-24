//! Controller trait for SDK-based agents.

use crate::core::SdkError;

/// Common lifecycle interface for SDK controllers.
///
/// Controllers wrap higher-level behavior (sensing/actuation) on top of FEAGI
/// networking and device registration.
pub trait Controller {
    /// Start the controller and any background loops.
    fn start(&mut self) -> Result<(), SdkError>;

    /// Stop the controller and release resources.
    fn stop(&mut self);

    /// Returns true if the controller is actively running.
    fn is_running(&self) -> bool;
}
