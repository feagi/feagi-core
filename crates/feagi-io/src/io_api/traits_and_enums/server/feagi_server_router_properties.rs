//! Factory trait for creating FeagiServerRouter instances.

use super::server_shared::FeagiServerBindStateChange;
use super::FeagiServerRouter;

/// Boxed callback type for server bind state changes.
pub type ServerStateChangeCallback =
    Box<dyn Fn(FeagiServerBindStateChange) + Send + Sync + 'static>;

/// Properties trait for creating a FeagiServerRouter instance.
///
/// Implement this trait to define the configuration needed to construct
/// a server router, then call `build()` to create the actual router.
pub trait FeagiServerRouterProperties {
    /// Build and return a boxed FeagiServerRouter instance.
    /// Consumes self to allow moving owned resources into the implementation.
    fn build(
        self: Box<Self>,
        state_change_callback: ServerStateChangeCallback,
    ) -> Box<dyn FeagiServerRouter>;
}
