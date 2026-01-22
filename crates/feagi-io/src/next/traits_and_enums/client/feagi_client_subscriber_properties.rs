//! Factory trait for creating FeagiClientSubscriber instances.

use super::FeagiClientSubscriber;
use super::client_shared::FeagiClientConnectionStateChange;

/// Properties trait for creating a FeagiClientSubscriber instance.
///
/// Implement this trait to define the configuration needed to construct
/// a client subscriber, then call `build()` to create the actual subscriber.
pub trait FeagiClientSubscriberProperties {
    /// Build and return a boxed FeagiClientSubscriber instance.
    /// Consumes self to allow moving owned resources into the implementation.
    fn build<F>(self, state_change_callback: F) -> Box<dyn FeagiClientSubscriber>
    where F: Fn(FeagiClientConnectionStateChange) + Send + Sync + 'static;
}
