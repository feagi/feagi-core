//! Tokio runtime helpers for async transports
//!
//! Provides utilities for managing tokio runtime in async transports.

use std::sync::Arc;
use tokio::runtime::{Builder, Runtime};

/// Runtime handle wrapper for shared access
pub type RuntimeHandle = Arc<Runtime>;

/// Create a new tokio runtime for async transports
///
/// # Configuration
/// - Multi-threaded runtime
/// - Worker threads = number of CPU cores
/// - Thread name prefix: "feagi-pns-async"
///
/// # Example
/// ```no_run
/// use feagi_pns::nonblocking::runtime;
///
/// let runtime = runtime::create_runtime().unwrap();
/// runtime.block_on(async {
///     // Async work here
/// });
/// ```
pub fn create_runtime() -> Result<Runtime, std::io::Error> {
    Builder::new_multi_thread()
        .worker_threads(num_cpus::get())
        .thread_name("feagi-pns-async")
        .enable_all()
        .build()
}

/// Create a runtime handle for shared access across threads
pub fn create_runtime_handle() -> Result<RuntimeHandle, std::io::Error> {
    create_runtime().map(Arc::new)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_creation() {
        let runtime = create_runtime().unwrap();
        runtime.block_on(async {
            // Simple async operation
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        });
    }

    #[test]
    fn test_runtime_handle() {
        let handle = create_runtime_handle().unwrap();
        handle.block_on(async {
            assert!(true);
        });
    }
}
