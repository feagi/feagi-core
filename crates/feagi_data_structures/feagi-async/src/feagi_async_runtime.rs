use core::future::Future;

pub trait FeagiAsyncRuntime: Send + Sync + 'static {
    /// The handle type returned by spawn - must be a future that yields T
    type TaskHandle<T: Send + 'static>: Future<Output = T> + Send + 'static;

    fn spawn<F, T>(&self, fut: F) -> Self::TaskHandle<T>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static;
}

/*
pub trait FeagiAsyncRuntime {
    type JoinHandle<T>: Future<Output = T> + 'static;

    fn spawn<F, T>(&self, fut: F) -> Self::JoinHandle<T>
    where
        F: Future<Output = T> + 'static,
        T: 'static;
}

 */