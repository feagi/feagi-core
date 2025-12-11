//! Basic example of using FeagiAsyncRuntime in a main program.
//!
//! This demonstrates the recommended pattern:
//! 1. Write your application logic as an async function taking &impl FeagiAsyncRuntime
//! 2. Use feagi_main! to generate the platform-specific entry point

use feagi_async::FeagiAsyncRuntime;

//region Application Logic

/// Example async function that does some "work"
async fn compute_answer(x: i32, y: i32) -> i32 {
    x + y
}

/// Another async function
async fn fetch_greeting(name: &str) -> String {
    format!("Hello, {}!", name)
}

/// The main application logic - takes runtime via trait
async fn run_application<R: FeagiAsyncRuntime>(runtime: &R) {
    println!("Application starting...");

    // Spawn concurrent tasks
    let handle1 = runtime.spawn(compute_answer(21, 21));
    let handle2 = runtime.spawn(fetch_greeting("FEAGI"));
    let handle3 = runtime.spawn(async {
        let values = vec![1, 2, 3, 4, 5];
        values.iter().sum::<i32>()
    });

    // Await all results
    let answer = handle1.await;
    let greeting = handle2.await;
    let sum = handle3.await;

    println!("Answer: {}", answer);       // 42
    println!("Greeting: {}", greeting);   // Hello, FEAGI!
    println!("Sum: {}", sum);             // 15

    println!("Application complete!");
}
//endregion

// Entry Point - ONE line, works on all platforms!
feagi_async::feagi_main!(run_application);
