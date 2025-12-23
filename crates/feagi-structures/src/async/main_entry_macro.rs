/// Generate a platform-specific async main entry point.
///
/// This macro creates the appropriate `main` function with the correct
/// async runtime attribute for the enabled platform feature.
///
/// # Usage
///
/// ```ignore
/// use feagi_structures::async::FeagiAsyncRuntime;
///
/// // Your application logic - platform agnostic
/// async fn run_application<R: FeagiAsyncRuntime>(runtime: &R) {
///     let handle = runtime.spawn(async { 42 });
///     println!("Result: {}", handle.await);
/// }
///
/// // Generate the main entry point
/// feagi_structures::feagi_main!(run_application);
/// ```
#[cfg(feature = "async")]
#[macro_export]
macro_rules! feagi_main {
    ($app_func:expr) => {
        // Check for conflicting features
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

        // Tokio entry point
        #[cfg(feature = "async-tokio")]
        #[tokio::main]
        async fn main() {
            $crate::async::run_async!($app_func);
        }

        // WASM entry point - must be sync, spawns async work
        #[cfg(feature = "async-wasm")]
        #[wasm_bindgen::prelude::wasm_bindgen(start)]
        pub fn main() {
            // Spawn the async application on the WASM event loop
            wasm_bindgen_futures::spawn_local(async {
                $crate::run_async!($app_func);
            });
        }

        // WASI entry point (if/when implemented)
        #[cfg(feature = "async-wasi")]
        fn main() {
            // WASI may need different handling - placeholder for now
            compile_error!("WASI main entry not yet implemented");
        }
    };
}
