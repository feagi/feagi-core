use crate::FeagiAsyncRuntime;

#[cfg(feature = "standard-tokio")]
use core::future::Future;
#[cfg(feature = "standard-tokio")]
use core::pin::Pin;
#[cfg(feature = "standard-tokio")]
use core::task::{Context, Poll};
#[cfg(feature = "standard-tokio")]
use tokio::runtime::Runtime;

/// The Tokio async runtime wrapper.
#[cfg(feature = "standard-tokio")]
pub struct TokioRuntime {
    runtime: Runtime,
}

#[cfg(feature = "standard-tokio")]
impl TokioRuntime {
    /// Create a new multi-threaded Tokio runtime.
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
