use crate::FeagiAsyncRuntime;

#[cfg(feature = "standard-tokio")]
use core::future::Future;
#[cfg(feature = "standard-tokio")]
use core::pin::Pin;
#[cfg(feature = "standard-tokio")]
use core::task::{Context, Poll};
#[cfg(feature = "standard-tokio")]
use tokio::runtime::{Handle, Runtime};

/// The Tokio async runtime wrapper that OWNS a runtime.
/// 
/// Use this when starting from a synchronous context (e.g., `main()`).
/// Do NOT use inside `#[tokio::test]` or other async contexts - use `TokioHandle` instead.
#[cfg(feature = "standard-tokio")]
pub struct TokioRuntime {
    runtime: Runtime,
}

#[cfg(feature = "standard-tokio")]
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

#[cfg(feature = "standard-tokio")]
impl Default for TokioRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "standard-tokio")]
impl FeagiAsyncRuntime for TokioRuntime {
    type TaskHandle<T: Send + 'static> = TokioTaskHandle<T>;

    fn spawn<F, T>(&self, fut: F) -> Self::TaskHandle<T>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        TokioTaskHandle(self.runtime.spawn(fut))
    }
}

#[cfg(feature = "standard-tokio")]
impl FeagiAsyncRuntime for TokioHandle {
    type TaskHandle<T: Send + 'static> = TokioTaskHandle<T>;

    fn spawn<F, T>(&self, fut: F) -> Self::TaskHandle<T>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        TokioTaskHandle(self.handle.spawn(fut))
    }
}

/// A handle to an existing Tokio runtime (does NOT own the runtime).
///
/// Use this when you're already inside a tokio async context
/// (e.g., inside `#[tokio::test]` or `#[tokio::main]`).
#[cfg(feature = "standard-tokio")]
pub struct TokioHandle {
    handle: Handle,
}

#[cfg(feature = "standard-tokio")]
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
#[cfg(feature = "standard-tokio")]
pub struct TokioTaskHandle<T>(tokio::task::JoinHandle<T>);

#[cfg(feature = "standard-tokio")]
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
#[cfg(feature = "standard-tokio")]
unsafe impl<T: Send> Send for TokioTaskHandle<T> {} // lol
