//! Factory trait for creating FeagiClientPusher instances.

use super::FeagiClientPusher;
use super::client_shared::FeagiClientConnectionStateChange;

/// Properties trait for creating a FeagiClientPusher instance.
///
/// Implement this trait to define the configuration needed to construct
/// a client pusher, then call `build()` to create the actual pusher.
pub trait FeagiClientPusherProperties {
    /// Build and return a boxed FeagiClientPusher instance.
    /// Consumes self to allow moving owned resources into the implementation.
    fn build<F>(self, state_change_callback: F) -> Box<dyn FeagiClientPusher>
    where F: Fn(FeagiClientConnectionStateChange) + Send + Sync + 'static;
}
