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
/// use feagi_data_structures::async::FeagiAsyncRuntime;
///
/// async fn my_async_work<R: FeagiAsyncRuntime>(runtime: &R) -> i32 {
///     let handle = runtime.spawn(async { 42 });
///     handle.await
/// }
///
/// #[tokio::test]  // or #[wasm_bindgen_test] on WASM
/// async fn test_something() {
///     let result = feagi_data_structures::run_async!(my_async_work);
///     assert_eq!(result, 42);
/// }
/// ```
#[cfg(feature = "async")]
#[macro_export]
macro_rules! run_async {
    ($func:expr) => {{
        // Check for conflicting feature combinations
        #[cfg(all(feature = "async-tokio", feature = "async-wasm"))]
        compile_error!("Do not enable both async-tokio and async-wasm features!");

        #[cfg(all(feature = "async-tokio", feature = "async-wasi"))]
        compile_error!("Do not enable both async-tokio and async-wasi features!");

        #[cfg(all(feature = "async-wasm", feature = "async-wasi"))]
        compile_error!("Do not enable both async-wasm and async-wasi features!");

        #[cfg(not(any(feature = "async-tokio", feature = "async-wasm", feature = "async-wasi")))]
        compile_error!(
            "No async runtime feature enabled! Enable one of: async-tokio, async-wasm, async-wasi"
        );

        // Get/create the appropriate runtime and run the function
        #[cfg(feature = "async-tokio")]
        {
            let runtime = $crate::r#async::TokioHandle::current();
            $func(&runtime).await
        }

        #[cfg(feature = "async-wasm")]
        {
            let runtime = $crate::r#async::WasmRuntime::new();
            $func(&runtime).await
        }

        #[cfg(feature = "async-wasi")]
        {
            let runtime = $crate::r#async::WasiHandle::current();
            $func(&runtime).await
        }
    }};
}
