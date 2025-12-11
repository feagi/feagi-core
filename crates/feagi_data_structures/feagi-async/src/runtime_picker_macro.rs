/// Outputs the correct async runtime implementation as defined by build settings.
/// Also causes a compile error if multiple runtimes are enabled at once.
#[macro_export]
macro_rules! runtime_picker {
    () => {{
        // Check for conflicting feature combinations
        #[cfg(all(feature = "standard-tokio", feature = "wasm"))]
        compile_error!("Do not enable both standard-tokio and wasm features!");

        #[cfg(all(feature = "standard-tokio", feature = "wasi"))]
        compile_error!("Do not enable both standard-tokio and wasi features!");

        #[cfg(all(feature = "wasm", feature = "wasi"))]
        compile_error!("Do not enable both wasm and wasi features!");

        // Return the appropriate runtime
        #[cfg(feature = "standard-tokio")]
        {
            $crate::TokioRuntime::new()
        }

        #[cfg(feature = "wasm")]
        {
            $crate::WasmRuntime::new()
        }

        #[cfg(feature = "wasi")]
        {
            $crate::WasiRuntime::new()
        }

        // Error if no runtime is enabled
        #[cfg(not(any(feature = "standard-tokio", feature = "wasm", feature = "wasi")))]
        compile_error!("No async runtime feature enabled! Enable one of: standard-tokio, wasm, wasi");
    }};
}