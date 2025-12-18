/// Generate a platform-specific async main entry point.
///
/// This macro creates the appropriate `main` function with the correct
/// async runtime attribute for the enabled platform feature.
///
/// # Usage
///
/// ```ignore
/// use feagi_async::FeagiAsyncRuntime;
///
/// // Your application logic - platform agnostic
/// async fn run_application<R: FeagiAsyncRuntime>(runtime: &R) {
///     let handle = runtime.spawn(async { 42 });
///     println!("Result: {}", handle.await);
/// }
///
/// // Generate the main entry point
/// feagi_async::feagi_main!(run_application);
/// ```
#[macro_export]
macro_rules! feagi_main {
    ($app_func:expr) => {
        // Check for conflicting features
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

        // Tokio entry point
        #[cfg(feature = "standard-tokio")]
        #[tokio::main]
        async fn main() {
            $crate::run_async!($app_func);
        }

        // WASM entry point - must be sync, spawns async work
        #[cfg(feature = "wasm")]
        #[wasm_bindgen::prelude::wasm_bindgen(start)]
        pub fn main() {
            // Spawn the async application on the WASM event loop
            wasm_bindgen_futures::spawn_local(async {
                $crate::run_async!($app_func);
            });
        }

        // WASI entry point (if/when implemented)
        #[cfg(feature = "wasi")]
        fn main() {
            // WASI may need different handling - placeholder for now
            compile_error!("WASI main entry not yet implemented");
        }
    };
}
