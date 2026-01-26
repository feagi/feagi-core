//! Factory trait for creating FeagiClientSubscriber instances.

use super::client_shared::FeagiClientConnectionStateChange;
use super::FeagiClientSubscriber;

/// Boxed callback type for client connection state changes.
pub type ClientStateChangeCallback =
    Box<dyn Fn(FeagiClientConnectionStateChange) + Send + Sync + 'static>;

/// Properties trait for creating a FeagiClientSubscriber instance.
///
/// Implement this trait to define the configuration needed to construct
/// a client subscriber, then call `build()` to create the actual subscriber.
pub trait FeagiClientSubscriberProperties {
    /// Build and return a boxed FeagiClientSubscriber instance.
    /// Consumes self to allow moving owned resources into the implementation.
    fn build(
        self: Box<Self>,
        state_change_callback: ClientStateChangeCallback,
    ) -> Box<dyn FeagiClientSubscriber>;
}
