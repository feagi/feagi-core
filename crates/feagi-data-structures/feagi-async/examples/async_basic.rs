//! Basic example of using FeagiAsyncRuntime in a main program.
//!
//! Run with: cargo run --example async_basic
//!
//! This demonstrates the recommended pattern:
//! 1. Write your application logic as an async function taking &impl FeagiAsyncRuntime
//! 2. Use feagi_main! to generate the platform-specific entry point

use feagi_async::FeagiAsyncRuntime;

//region Application Logic (only compiled for tokio)

async fn compute_answer(x: i32, y: i32) -> i32 {
    x + y
}

async fn fetch_greeting(name: &str) -> String {
    format!("Hello, {}!", name)
}

async fn run_application<R: FeagiAsyncRuntime>(runtime: &R) {
    use std::time::Duration;
    
    println!("Application starting...");

    let handle1 = runtime.spawn(compute_answer(21, 21));
    let handle2 = runtime.spawn(fetch_greeting("FEAGI"));
    let handle3 = runtime.spawn(async {
        let values = vec![1, 2, 3, 4, 5];
        values.iter().sum::<i32>()
    });

    let answer = handle1.await;
    let greeting = handle2.await;
    let sum = handle3.await;

    println!("Answer: {}", answer);       // 42
    println!("Greeting: {}", greeting);   // Hello, FEAGI!
    println!("Sum: {}", sum);             // 15

    // Demonstrate delay() - platform-agnostic delay
    println!("Delaying for 100ms...");
    runtime.delay(Duration::from_millis(100)).await;
    println!("Delay complete!");

    println!("Application complete!");
}
//endregion

// Note: due to rust testing arch, we have this feature check, DO NOT USE THIS IN YOUR ACTUAL PROGRAM!
#[cfg(feature = "standard-tokio")]
feagi_async::feagi_main!(run_application);

// Fallback main for when compiled without standard-tokio (e.g., during wasm-pack test)
#[cfg(not(feature = "standard-tokio"))]
fn main() {
    // This example requires --features standard-tokio
    // WASM apps should be built as separate web projects
}
