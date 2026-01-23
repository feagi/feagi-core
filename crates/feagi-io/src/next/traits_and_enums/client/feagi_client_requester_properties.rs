//! Factory trait for creating FeagiClientRequester instances.

use super::FeagiClientRequester;
use super::client_shared::FeagiClientConnectionStateChange;

/// Boxed callback type for client connection state changes.
pub type ClientStateChangeCallback = Box<dyn Fn(FeagiClientConnectionStateChange) + Send + Sync + 'static>;

/// Properties trait for creating a FeagiClientRequester instance.
///
/// Implement this trait to define the configuration needed to construct
/// a client requester, then call `build()` to create the actual requester.
pub trait FeagiClientRequesterProperties {
    /// Build and return a boxed FeagiClientRequester instance.
    /// Consumes self to allow moving owned resources into the implementation.
    fn build(self: Box<Self>, state_change_callback: ClientStateChangeCallback) -> Box<dyn FeagiClientRequester>;
}
