//! Factory trait for creating FeagiServerPuller instances.

use super::FeagiServerPuller;
use super::server_shared::FeagiServerBindStateChange;

/// Properties trait for creating a FeagiServerPuller instance.
///
/// Implement this trait to define the configuration needed to construct
/// a server puller, then call `build()` to create the actual puller.
pub trait FeagiServerPullerProperties {
    /// Build and return a boxed FeagiServerPuller instance.
    /// Consumes self to allow moving owned resources into the implementation.
    fn build<F>(self, state_change_callback: F) -> Box<dyn FeagiServerPuller>
    where F: Fn(FeagiServerBindStateChange) + Send + Sync + 'static;
}
