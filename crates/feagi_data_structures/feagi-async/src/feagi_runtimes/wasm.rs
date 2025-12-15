use crate::FeagiAsyncRuntime;

#[cfg(feature = "wasm")]
use core::future::Future;
#[cfg(feature = "wasm")]
use core::pin::Pin;
#[cfg(feature = "wasm")]
use core::task::{Context, Poll};
#[cfg(feature = "wasm")]
use core::time::Duration;
#[cfg(feature = "wasm")]
use futures_channel::oneshot;
#[cfg(feature = "wasm")]
use futures_util::future::select;
#[cfg(feature = "wasm")]
use futures_util::FutureExt;
#[cfg(feature = "wasm")]
use wasm_bindgen_futures::{spawn_local, JsFuture};
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;
#[cfg(feature = "wasm")]
use js_sys::Promise;
#[cfg(feature = "wasm")]
use web_sys::Window;

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

/// A Send-safe wrapper for WASM delay future
/// 
/// In WASM, everything runs on a single thread, so Send is trivially satisfied.
/// This wrapper makes JsFuture Send-safe for use in the trait.
#[cfg(feature = "wasm")]
struct WasmDelayFuture {
    inner: JsFuture,
}

#[cfg(feature = "wasm")]
impl Future for WasmDelayFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match Pin::new(&mut self.inner).poll(cx) {
            Poll::Ready(Ok(_)) => Poll::Ready(()),
            Poll::Ready(Err(_)) => Poll::Ready(()), // Ignore errors
            Poll::Pending => Poll::Pending,
        }
    }
}

// SAFETY: In WASM single-threaded environment, Send is satisfied trivially.
#[cfg(feature = "wasm")]
unsafe impl Send for WasmDelayFuture {}

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

    fn delay(&self, duration: Duration) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>> {
        // Use setTimeout via Promise for WASM
        let millis = duration.as_millis() as u32;
        let promise = Promise::new(&mut |resolve, _reject| {
            let window = web_sys::window().expect("window should be available");
            let closure = Closure::once_into_js(move || {
                resolve.call0(&JsValue::NULL).unwrap();
            });
            window
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    closure.as_ref().unchecked_ref(),
                    millis as i32,
                )
                .expect("setTimeout should work");
            // closure is kept alive by the Promise
        });
        
        // Wrap JsFuture in a Send-safe wrapper
        let delay_future = WasmDelayFuture {
            inner: JsFuture::from(promise),
        };
        
        Box::pin(delay_future)
    }

    fn try_block_on<F, T>(&self, _future: F) -> Result<T, crate::BlockOnError>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        Err(crate::BlockOnError::not_supported(
            "WASM does not support blocking operations. All operations must be async."
        ))
    }

    fn with_timeout<F, T>(
        &self,
        future: F,
        timeout: Duration,
    ) -> Pin<Box<dyn Future<Output = Result<T, crate::TimeoutError>> + Send + 'static>>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        // Create delay future before moving into async block
        let delay = self.delay(timeout);
        Box::pin(async move {
            match select(future.boxed(), delay).await {
                futures_util::future::Either::Left((result, _)) => Ok(result),
                futures_util::future::Either::Right((_, _)) => Err(crate::TimeoutError),
            }
        })
    }
}