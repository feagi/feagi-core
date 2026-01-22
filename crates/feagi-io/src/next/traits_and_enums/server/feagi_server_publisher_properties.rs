//! Factory trait for creating FeagiServerPublisher instances.

use super::FeagiServerPublisher;
use super::server_shared::FeagiServerBindStateChange;

/// Properties trait for creating a FeagiServerPublisher instance.
///
/// Implement this trait to define the configuration needed to construct
/// a server publisher, then call `build()` to create the actual publisher.
pub trait FeagiServerPublisherProperties {
    /// Build and return a boxed FeagiServerPublisher instance.
    /// Consumes self to allow moving owned resources into the implementation.
    fn build<F>(self, state_change_callback: F) -> Box<dyn FeagiServerPublisher>
    where F: Fn(FeagiServerBindStateChange) + Send + Sync + 'static;
}
