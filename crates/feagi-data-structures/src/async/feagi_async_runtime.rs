use core::future::Future;
use core::pin::Pin;
use core::time::Duration;

/// Error returned when blocking is not supported (e.g., in WASM)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockOnError {
    pub message: String,
}

impl BlockOnError {
    pub fn not_supported(reason: &str) -> Self {
        Self {
            message: format!("Blocking not supported: {}", reason),
        }
    }
}

impl std::fmt::Display for BlockOnError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for BlockOnError {}

/// Error returned when a timeout expires
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeoutError;

impl std::fmt::Display for TimeoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Operation timed out")
    }
}

impl std::error::Error for TimeoutError {}

pub trait FeagiAsyncRuntime: Send + Sync + 'static {
    /// The handle type returned by spawn - must be a future that yields T
    type TaskHandle<T: Send + 'static>: Future<Output = T> + Send + 'static;

    fn spawn<F, T>(&self, fut: F) -> Self::TaskHandle<T>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static;

    /// Delay execution for a specified duration
    ///
    /// Returns a future that completes after the specified duration.
    /// This is platform-agnostic:
    /// - Desktop: Uses tokio::time::sleep
    /// - WASM: Uses setTimeout via wasm-bindgen
    ///
    /// # Arguments
    ///
    /// * `duration` - The duration to delay
    ///
    /// # Returns
    ///
    /// A future that completes after the duration
    fn delay(&self, duration: Duration) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

    /// Attempt to block on a future until it completes
    ///
    /// This is platform-specific:
    /// - Desktop: Blocks the current thread until the future completes
    /// - WASM: Returns an error (blocking is not supported in WASM)
    ///
    /// # Arguments
    ///
    /// * `future` - The future to block on
    ///
    /// # Returns
    ///
    /// `Ok(T)` if blocking succeeded and future completed,
    /// `Err(BlockOnError)` if blocking is not supported or failed
    fn try_block_on<F, T>(&self, future: F) -> Result<T, BlockOnError>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static;

    /// Add a timeout to a future
    ///
    /// Returns `Ok(T)` if the future completes before the timeout,
    /// `Err(TimeoutError)` if the timeout expires first.
    ///
    /// This is platform-agnostic and uses `delay()` internally.
    ///
    /// # Arguments
    ///
    /// * `future` - The future to add a timeout to
    /// * `timeout` - The maximum duration to wait
    ///
    /// # Returns
    ///
    /// A future that resolves to `Result<T, TimeoutError>`
    fn with_timeout<F, T>(
        &self,
        future: F,
        timeout: Duration,
    ) -> Pin<Box<dyn Future<Output = Result<T, TimeoutError>> + Send + 'static>>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static;
}
