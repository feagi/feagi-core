//! Factory trait for creating FeagiClientRequester instances.

use super::FeagiClientRequester;
use super::client_shared::FeagiClientConnectionStateChange;

/// Properties trait for creating a FeagiClientRequester instance.
///
/// Implement this trait to define the configuration needed to construct
/// a client requester, then call `build()` to create the actual requester.
pub trait FeagiClientRequesterProperties {
    /// Build and return a boxed FeagiClientRequester instance.
    /// Consumes self to allow moving owned resources into the implementation.
    fn build<F>(self, state_change_callback: F) -> Box<dyn FeagiClientRequester>
    where F: Fn(FeagiClientConnectionStateChange) + Send + Sync + 'static;
}
