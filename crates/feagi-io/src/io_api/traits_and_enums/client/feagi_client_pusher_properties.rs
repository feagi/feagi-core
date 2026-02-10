//! Factory trait for creating FeagiClientPusher instances.

use super::client_shared::FeagiClientConnectionStateChange;
use super::FeagiClientPusher;

/// Boxed callback type for client connection state changes.
pub type ClientStateChangeCallback =
    Box<dyn Fn(FeagiClientConnectionStateChange) + Send + Sync + 'static>;

/// Properties trait for creating a FeagiClientPusher instance.
///
/// Implement this trait to define the configuration needed to construct
/// a client pusher, then call `build()` to create the actual pusher.
pub trait FeagiClientPusherProperties {
    /// Build and return a boxed FeagiClientPusher instance.
    /// Consumes self to allow moving owned resources into the implementation.
    fn build(
        self: Box<Self>,
        state_change_callback: ClientStateChangeCallback,
    ) -> Box<dyn FeagiClientPusher>;
}
