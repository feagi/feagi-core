
/// Outputs the correct async runtime implementation as defined by build settings.
/// Also causes a compile error if multiple runtimes are enabled at once
#[macro_export]
macro_rules! runtime_picker  {
    () => {
        #[cfg(all(feature = "standard-tokio", feature = "wasm"))]
        {
            compile_error!("Do not enable both standard-tokio and wasm features!");
        }

        #[cfg(all(feature = "standard-tokio", feature = "wasi"))]
        {
            compile_error!("Do not enable both standard-tokio and wasi features!");
        }

        #[cfg(all(feature = "wasm", feature = "wasi"))]
        {
            compile_error!("Do not enable both wasm and wasi features!");
        }

        #[cfg(feature = "standard-tokio")]
        {
            use feagi_async::{FeagiAsyncRuntime, TokioRuntime};
            feagi_async::TokioRuntime::new();
        }

        #[cfg(feature = "wasm")]
        {
            use feagi_async::{FeagiAsyncRuntime, WasmRuntime};
            feagi_async::WasmRuntime::new();
        }

        compile_error!("No Async runtime defined!");
    };
}