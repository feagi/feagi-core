use crate::FeagiAsyncRuntime;

#[cfg(feature = "standard-tokio")]
use tokio::runtime::Runtime;

#[cfg(feature = "standard-tokio")]
pub struct TokioRuntime {
    runtime: Runtime,
}

#[cfg(feature = "standard-tokio")]
impl TokioRuntime {
    pub fn new() -> Self {
        Self {
            runtime: Runtime::new().expect("Tokio runtime failed to initialize"),
        }
    }
}

#[cfg(feature = "standard-tokio")]
impl FeagiAsyncRuntime for TokioRuntime {
    type JoinHandle<T> = ();

    fn spawn<F>(future: F)
    where
        F: core::future::Future<Output=()> + Send + 'static,
    {
        tokio::spawn(future);
    }
}