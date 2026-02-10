//! Factory trait for creating FeagiServerPuller instances.

use super::server_shared::FeagiServerBindStateChange;
use super::FeagiServerPuller;

/// Boxed callback type for server bind state changes.
pub type ServerStateChangeCallback =
    Box<dyn Fn(FeagiServerBindStateChange) + Send + Sync + 'static>;

/// Properties trait for creating a FeagiServerPuller instance.
///
/// Implement this trait to define the configuration needed to construct
/// a server puller, then call `build()` to create the actual puller.
pub trait FeagiServerPullerProperties {
    /// Build and return a boxed FeagiServerPuller instance.
    /// Consumes self to allow moving owned resources into the implementation.
    fn build(
        self: Box<Self>,
        state_change_callback: ServerStateChangeCallback,
    ) -> Box<dyn FeagiServerPuller>;
}
