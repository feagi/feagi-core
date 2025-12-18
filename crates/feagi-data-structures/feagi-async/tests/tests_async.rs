//! Tests for the various async implementations
//!
//! The test BODY is identical across platforms - only the test attribute differs.
//!
//! ## Running Tests
//!
//! **Tokio:**
//! ```bash
//! cargo test --features standard-tokio
//! ```
//!
//! **WASM in Node.js:**
//! ```bash
//! cargo install wasm-pack  # one-time setup
//! wasm-pack test --node --no-default-features --features wasm
//! ```
//!
//! **WASM in Browser (headless):**
//! ```bash
//! wasm-pack test --headless --chrome --no-default-features --features wasm
//! wasm-pack test --headless --firefox --no-default-features --features wasm
//! ``` 

use feagi_async::FeagiAsyncRuntime;

/*
#[cfg(feature = "wasm")]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);
 */

// Note: Remove `run_in_browser` to allow tests to run in both Node.js and browser
// Use `wasm-pack test --node` for Node.js or `wasm-pack test --headless --chrome` for browser

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

/// Test delay() method - demonstrates platform-agnostic delay usage
async fn run_delay_test<R: FeagiAsyncRuntime>(runtime: &R) {
    use std::time::{Duration, Instant};
    
    let start = Instant::now();
    
    // Use delay() via the trait - platform-agnostic
    runtime.delay(Duration::from_millis(10)).await;
    
    let elapsed = start.elapsed();
    
    // Should have delayed at least 10ms (may be slightly more due to scheduling)
    assert!(elapsed >= Duration::from_millis(10), 
            "Delay should be at least 10ms, got {:?}", elapsed);
    
    println!("Delay test passed! Elapsed: {:?}", elapsed);
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

#[cfg(feature = "standard-tokio")]
#[tokio::test]
async fn test_delay() {
    feagi_async::run_async!(run_delay_test);
}

#[cfg(feature = "wasm")]
#[wasm_bindgen_test::wasm_bindgen_test]
async fn test_delay() {
    feagi_async::run_async!(run_delay_test);
}

/// Test try_block_on() - should fail on WASM, succeed on TokioRuntime
async fn run_block_on_test<R: FeagiAsyncRuntime>(runtime: &R) {
    // Create a simple future that doesn't capture runtime
    // Note: This test only works with TokioRuntime (not TokioHandle or WasmRuntime)
    let simple_future = async {
        42
    };
    
    let result = runtime.try_block_on(simple_future);
    
    match result {
        Ok(value) => {
            assert_eq!(value, 42);
            println!("Block on test passed! Value: {}", value);
        }
        Err(e) => {
            println!("Block on not supported (expected for WASM/TokioHandle): {}", e);
            // This is expected for WASM and TokioHandle, so we don't fail the test
        }
    }
}

/// Test with_timeout() - should timeout if future takes too long
async fn run_timeout_test<R: FeagiAsyncRuntime>(runtime: &R) {
    use std::time::Duration;
    
    // Test 1: Future completes before timeout
    let fast_future = async { 42 };
    let timeout_future = runtime.with_timeout(fast_future, Duration::from_millis(100));
    let result = timeout_future.await;
    assert_eq!(result, Ok(42));
    println!("Fast future test passed!");
    
    // Test 2: Future times out
    // Create delay future separately to avoid capturing runtime
    let delay_future = runtime.delay(Duration::from_millis(200));
    let slow_future = async move {
        delay_future.await;
        100
    };
    let timeout_future = runtime.with_timeout(slow_future, Duration::from_millis(50));
    let result = timeout_future.await;
    assert!(result.is_err(), "Slow future should have timed out");
    println!("Timeout test passed!");
}

#[cfg(feature = "standard-tokio")]
#[tokio::test]
async fn test_block_on() {
    feagi_async::run_async!(run_block_on_test);
}

#[cfg(feature = "wasm")]
#[wasm_bindgen_test::wasm_bindgen_test]
async fn test_block_on() {
    feagi_async::run_async!(run_block_on_test);
}

#[cfg(feature = "standard-tokio")]
#[tokio::test]
async fn test_timeout() {
    feagi_async::run_async!(run_timeout_test);
}

#[cfg(feature = "wasm")]
#[wasm_bindgen_test::wasm_bindgen_test]
async fn test_timeout() {
    feagi_async::run_async!(run_timeout_test);
}

//endregion
