//! Tests for the various async implementations
//!
//! The test BODY is identical across platforms - only the entry point differs.
//! This demonstrates how downstream libraries can write platform-agnostic async code.

use feagi_async::{runtime_picker, FeagiAsyncRuntime};

//region Shared Tests
async fn async_number_test(x: i32, y: i32) -> i32 {
    x + y
}

async fn async_string_formatting(name: String) -> String {
    format!("Hello, {}!", name)
}

async fn run_test_logic<R: FeagiAsyncRuntime>(runtime: &R) {
    // Spawn an async task that returns a value
    let handle1 = runtime.spawn(async_number_test(21, 21));

    // Spawn another task
    let handle2 = runtime.spawn(async_string_formatting("FEAGI".to_string()));

    // Spawn a task with a closure
    let handle3 = runtime.spawn(async {
        let values = vec![1, 2, 3, 4, 5];
        values.iter().sum::<i32>()
    });

    // Await all the results
    let answer = handle1.await;
    let greeting = handle2.await;
    let sum = handle3.await;

    assert_eq!(answer, 42);
    assert_eq!(greeting, String::from("Hello, FEAGI!"));
    assert_eq!(sum, 15);
}
//endregion


/// Macro that generates a test with the correct async entry point per platform.
/// The test BODY is the same - only the wrapper differs.
macro_rules! feagi_async_test {
    ($name:ident, $test_body:expr) => {
        #[cfg(feature = "standard-tokio")]
        #[tokio::test]
        async fn $name() {
            let runtime = runtime_picker!();
            $test_body(&runtime).await;
        }

        #[cfg(feature = "wasm")]
        #[wasm_bindgen_test::wasm_bindgen_test]
        async fn $name() {
            let runtime = runtime_picker!();
            $test_body(&runtime).await;
        }
    };
}


feagi_async_test!(test_spawn_and_await, run_test_logic);
