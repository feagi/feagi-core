use crate::FeagiAsyncRuntime;

pub struct WasmRuntime;

impl AsyncRuntime for WasmRuntime {
    fn spawn<F>(future: F)
    where
        F: core::future::Future<Output = ()> + 'static,
    {
        // spawn_local is non-Send and okay for wasm
        spawn_local(future);
    }

    fn block_on<F: core::future::Future>(_future: F) -> F::Output {
        panic!("block_on not supported on wasm");
    }
}