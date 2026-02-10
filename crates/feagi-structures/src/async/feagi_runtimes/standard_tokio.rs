use super::super::FeagiAsyncRuntime;

#[cfg(feature = "async-tokio")]
use core::future::Future;
#[cfg(feature = "async-tokio")]
use core::pin::Pin;
#[cfg(feature = "async-tokio")]
use core::task::{Context, Poll};
#[cfg(feature = "async-tokio")]
use core::time::Duration;
#[cfg(feature = "async-tokio")]
use tokio::runtime::{Handle, Runtime};

/// The Tokio async runtime wrapper that OWNS a runtime.
///
/// Use this when starting from a synchronous context (e.g., `main()`).
/// Do NOT use inside `#[tokio::test]` or other async contexts - use `TokioHandle` instead.
#[cfg(feature = "async-tokio")]
pub struct TokioRuntime {
    runtime: Runtime,
}

#[cfg(feature = "async-tokio")]
impl TokioRuntime {
    /// Create a new multi-threaded Tokio runtime.
    ///
    /// Use this from synchronous code (e.g., `main()`).
    /// If you're already inside an async context, use `TokioHandle::current()` instead.
    pub fn new() -> Self {
        Self {
            runtime: Runtime::new().expect("Tokio runtime failed to initialize"),
        }
    }

    /// Run a future to completion on this runtime.
    ///
    /// This blocks the current thread until the future completes.
    pub fn block_on<F: Future>(&self, fut: F) -> F::Output {
        self.runtime.block_on(fut)
    }
}

#[cfg(feature = "async-tokio")]
impl Default for TokioRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "async-tokio")]
impl FeagiAsyncRuntime for TokioRuntime {
    type TaskHandle<T: Send + 'static> = TokioTaskHandle<T>;

    fn spawn<F, T>(&self, fut: F) -> Self::TaskHandle<T>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        TokioTaskHandle(self.runtime.spawn(fut))
    }

    fn delay(&self, duration: Duration) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>> {
        Box::pin(tokio::time::sleep(duration))
    }

    fn try_block_on<F, T>(&self, future: F) -> Result<T, super::super::BlockOnError>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        Ok(self.runtime.block_on(future))
    }

    fn with_timeout<F, T>(
        &self,
        future: F,
        timeout: Duration,
    ) -> Pin<Box<dyn Future<Output = Result<T, super::super::TimeoutError>> + Send + 'static>>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        Box::pin(async move {
            tokio::select! {
                result = future => Ok(result),
                _ = tokio::time::sleep(timeout) => Err(super::super::TimeoutError),
            }
        })
    }
}

#[cfg(feature = "async-tokio")]
impl FeagiAsyncRuntime for TokioHandle {
    type TaskHandle<T: Send + 'static> = TokioTaskHandle<T>;

    fn spawn<F, T>(&self, fut: F) -> Self::TaskHandle<T>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        TokioTaskHandle(self.handle.spawn(fut))
    }

    fn delay(&self, duration: Duration) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>> {
        Box::pin(tokio::time::sleep(duration))
    }

    fn try_block_on<F, T>(&self, _future: F) -> Result<T, super::super::BlockOnError>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        // TokioHandle doesn't own a runtime, so we can't block_on directly
        // We need to spawn the future and wait for it
        // However, this is tricky without blocking - for now, return an error
        // indicating that blocking requires TokioRuntime (not TokioHandle)
        Err(super::super::BlockOnError::not_supported(
            "TokioHandle does not support blocking. Use TokioRuntime::try_block_on() instead.",
        ))
    }

    fn with_timeout<F, T>(
        &self,
        future: F,
        timeout: Duration,
    ) -> Pin<Box<dyn Future<Output = Result<T, super::super::TimeoutError>> + Send + 'static>>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        Box::pin(async move {
            tokio::select! {
                result = future => Ok(result),
                _ = tokio::time::sleep(timeout) => Err(super::super::TimeoutError),
            }
        })
    }
}

/// A handle to an existing Tokio runtime (does NOT own the runtime).
///
/// Use this when you're already inside a tokio async context
/// (e.g., inside `#[tokio::test]` or `#[tokio::main]`).
#[cfg(feature = "async-tokio")]
pub struct TokioHandle {
    handle: Handle,
}

#[cfg(feature = "async-tokio")]
impl TokioHandle {
    /// Get a handle to the current tokio runtime.
    ///
    /// Panics if not called from within a tokio runtime context.
    /// Use this inside `#[tokio::test]`, `#[tokio::main]`, or any async code
    /// running on tokio.
    pub fn current() -> Self {
        Self {
            handle: Handle::current(),
        }
    }
}

/// A wrapper around Tokio's `JoinHandle` that implements `Future<Output = T>`.
///
/// Tokio's native `JoinHandle<T>` returns `Result<T, JoinError>`, but FeagiAsyncRuntime
/// requires `Future<Output = T>`. This wrapper unwraps the result and panics
/// if the spawned task panicked.
#[cfg(feature = "async-tokio")]
pub struct TokioTaskHandle<T>(tokio::task::JoinHandle<T>);

#[cfg(feature = "async-tokio")]
impl<T> Future for TokioTaskHandle<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
        // We're projecting through the newtype to poll the inner JoinHandle.
        // This is safe because we don't move the inner value.
        let inner = unsafe { self.map_unchecked_mut(|s| &mut s.0) };
        match inner.poll(cx) {
            Poll::Ready(Ok(value)) => Poll::Ready(value),
            Poll::Ready(Err(e)) => panic!("Spawned task panicked: {e}"),
            Poll::Pending => Poll::Pending,
        }
    }
}

// TokioTaskHandle is Send if T is Send, which matches tokio::task::JoinHandle's behavior
#[cfg(feature = "async-tokio")]
unsafe impl<T: Send> Send for TokioTaskHandle<T> {} // lol
