/// Run an async function with the platform-appropriate runtime.
///
/// Creates or gets a handle to the runtime based on the enabled feature,
/// calls the provided async function, and awaits the result.
///
/// Use this inside any async context (`#[tokio::test]`, `#[tokio::main]`,
/// `#[wasm_bindgen_test]`, etc.).
///
/// # Usage
///
/// ```ignore
/// use feagi_async::FeagiAsyncRuntime;
///
/// async fn my_async_work<R: FeagiAsyncRuntime>(runtime: &R) -> i32 {
///     let handle = runtime.spawn(async { 42 });
///     handle.await
/// }
///
/// #[tokio::test]  // or #[wasm_bindgen_test] on WASM
/// async fn test_something() {
///     let result = feagi_async::run_async!(my_async_work);
///     assert_eq!(result, 42);
/// }
/// ```
#[macro_export]
macro_rules! run_async {
    ($func:expr) => {{
        // Check for conflicting feature combinations
        #[cfg(all(feature = "standard-tokio", feature = "wasm"))]
        compile_error!("Do not enable both standard-tokio and wasm features!");

        #[cfg(all(feature = "standard-tokio", feature = "wasi"))]
        compile_error!("Do not enable both standard-tokio and wasi features!");

        #[cfg(all(feature = "wasm", feature = "wasi"))]
        compile_error!("Do not enable both wasm and wasi features!");

        #[cfg(not(any(feature = "standard-tokio", feature = "wasm", feature = "wasi")))]
        compile_error!(
            "No async runtime feature enabled! Enable one of: standard-tokio, wasm, wasi"
        );

        // Get/create the appropriate runtime and run the function
        #[cfg(feature = "standard-tokio")]
        {
            let runtime = $crate::TokioHandle::current();
            $func(&runtime).await
        }

        #[cfg(feature = "wasm")]
        {
            let runtime = $crate::WasmRuntime::new();
            $func(&runtime).await
        }

        #[cfg(feature = "wasi")]
        {
            let runtime = $crate::WasiHandle::current();
            $func(&runtime).await
        }
    }};
}
