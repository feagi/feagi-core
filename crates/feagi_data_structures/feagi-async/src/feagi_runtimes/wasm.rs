use crate::FeagiAsyncRuntime;

#[cfg(feature = "wasm")]
use core::future::Future;
#[cfg(feature = "wasm")]
use core::pin::Pin;
#[cfg(feature = "wasm")]
use core::task::{Context, Poll};
#[cfg(feature = "wasm")]
use futures_channel::oneshot;
#[cfg(feature = "wasm")]
use wasm_bindgen_futures::spawn_local;

/// The WASM async runtime using wasm-bindgen-futures.
/// 
/// This is a single-threaded runtime suitable for browser and web environments.
/// Note: There is no `block_on` equivalent in WASM - everything must be async.
#[cfg(feature = "wasm")]
pub struct WasmRuntime;

#[cfg(feature = "wasm")]
impl WasmRuntime {
    /// Create a new WASM runtime.
    pub fn new() -> Self {
        Self
    }
}

#[cfg(feature = "wasm")]
impl Default for WasmRuntime {
    fn default() -> Self {
        Self::new()
    }
}

/// A task handle for WASM that wraps a oneshot channel receiver.
/// 
/// Since `spawn_local` doesn't return a handle, we use a channel internally
/// to communicate the result back to the caller.
#[cfg(feature = "wasm")]
pub struct WasmTaskHandle<T>(oneshot::Receiver<T>);

#[cfg(feature = "wasm")]
impl<T> Future for WasmTaskHandle<T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
        // Poll the oneshot receiver
        match Pin::new(&mut self.0).poll(cx) {
            Poll::Ready(Ok(value)) => Poll::Ready(value),
            Poll::Ready(Err(_)) => panic!("Spawned task was cancelled (sender dropped)"),
            Poll::Pending => Poll::Pending,
        }
    }
}

// SAFETY: In WASM single-threaded environment, Send is satisfied trivially.
// The oneshot channel from futures-channel is Send when T is Send.
#[cfg(feature = "wasm")]
unsafe impl<T: Send> Send for WasmTaskHandle<T> {}

#[cfg(feature = "wasm")]
impl FeagiAsyncRuntime for WasmRuntime {
    type TaskHandle<T: Send + 'static> = WasmTaskHandle<T>;

    fn spawn<F, T>(&self, fut: F) -> Self::TaskHandle<T>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        let (tx, rx) = oneshot::channel();
        
        // spawn_local doesn't require Send on the future in WASM,
        // but we accept Send futures to match the trait signature
        spawn_local(async move {
            let result = fut.await;
            // Ignore send errors - receiver may have been dropped
            let _ = tx.send(result);
        });
        
        WasmTaskHandle(rx)
    }
}