//! Tests for the various async implementations
//!
//! The test BODY is identical across platforms - only the test attribute differs.

use feagi_async::FeagiAsyncRuntime;

//region Shared Tests
async fn async_number_test(x: i32, y: i32) -> i32 {
    x + y
}

async fn async_string_formatting(name: String) -> String {
    format!("Hello, {}!", name)
}

/// The actual test logic - takes a runtime via the trait.
/// This is what downstream library code looks like: pure async, no platform specifics.
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
    println!("Simple Async Tests Done!");
}

//endregion

//region Call tests per platform

#[cfg(feature = "standard-tokio")]
#[tokio::test]
async fn test_spawn_and_await() {
    feagi_async::run_async!(run_test_logic);
}

#[cfg(feature = "wasm")]
#[wasm_bindgen_test::wasm_bindgen_test]
async fn test_spawn_and_await() {
    feagi_async::run_async!(run_test_logic);
}
//endregion
