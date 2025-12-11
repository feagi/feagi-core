use core::future::Future;

pub trait FeagiAsyncRuntime {
    type JoinHandle<T>: Future<Output = T> + 'static;

    fn spawn<F, T>(&self, fut: F) -> Self::JoinHandle<T>
    where
        F: Future<Output = T> + 'static,
        T: 'static;
}