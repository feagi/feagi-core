//! Basic example of using FeagiAsyncRuntime with Tokio.
//!
//! Run with: cargo run --example tokio_basic --features standard-tokio

use feagi_async::{FeagiAsyncRuntime, TokioRuntime};

/// An example async function that does some "work" and returns a value.
async fn compute_answer(x: i32, y: i32) -> i32 {
    x + y
}

/// Another async function that returns a String.
async fn fetch_greeting(name: String) -> String {
    format!("Hello, {}!", name)
}

fn main() {
    // Create the Tokio runtime through our abstraction
    let runtime = TokioRuntime::new();

    // Use block_on to run async code from a sync context
    // Note: We need to enter the runtime context to await spawned tasks
    runtime.block_on(async {
        // Spawn an async task that returns a value
        let handle1 = runtime.spawn(compute_answer(21, 21));

        // Spawn another task
        let handle2 = runtime.spawn(fetch_greeting("FEAGI".to_string()));

        // Spawn a task with a closure
        let handle3 = runtime.spawn(async {
            let values = vec![1, 2, 3, 4, 5];
            // Simulate async processing
            tokio::time::sleep(std::time::Duration::from_millis(75)).await;
            values.iter().sum::<i32>()
        });

        // Await all the results
        let answer = handle1.await;
        let greeting = handle2.await;
        let sum = handle3.await;

        println!("Answer: {}", answer);       // Answer: 42
        println!("Greeting: {}", greeting);   // Greeting: Hello, FEAGI!
        println!("Sum: {}", sum);             // Sum: 15
    });

    println!("All tasks completed!");
}

