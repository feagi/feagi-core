//! Factory trait for creating FeagiServerRouter instances.

use super::FeagiServerRouter;
use super::server_shared::FeagiServerBindStateChange;

/// Properties trait for creating a FeagiServerRouter instance.
///
/// Implement this trait to define the configuration needed to construct
/// a server router, then call `build()` to create the actual router.
pub trait FeagiServerRouterProperties {
    /// Build and return a boxed FeagiServerRouter instance.
    /// Consumes self to allow moving owned resources into the implementation.
    fn build<F>(self, state_change_callback: F) -> Box<dyn FeagiServerRouter>
    where F: Fn(FeagiServerBindStateChange) + Send + Sync + 'static;
}
