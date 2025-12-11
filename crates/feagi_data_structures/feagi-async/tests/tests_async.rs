//! Tests for the various async implementations

macro_rules! simple_async_test {
    ($runtime:expr) => {
        let runtime = $runtime;
        runtime.block_on(async {
            // Spawn an async task that returns a value
            let handle1 = runtime.spawn(async_number_test(21, 21));

            // Spawn another task
            let handle2 = runtime.spawn(async_string_formatting("FEAGI".to_string()));

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

            assert_eq!(answer, 42);
            assert_eq!(greeting, String::from("Hello, FEAGI!"));
            assert_eq!(sum, 15);
        });
    };
}

async fn async_number_test(x: i32, y: i32) -> i32 {
    x + y
}

async fn async_string_formatting(name: String) -> String {
    format!("Hello, {}!", name)
}



#[cfg(test)]
#[cfg(feature = "standard-tokio")]
mod tests_tokio {
    use feagi_async::{runtime_picker, FeagiAsyncRuntime};
    use crate::{async_number_test, async_string_formatting};

    #[test]
    fn simple_tokio_test() {
        let runtime = runtime_picker!();
        simple_async_test!(runtime);
    }

}

#[cfg(test)]
#[cfg(feature = "wasm")]
mod tests_wasm {
    use feagi_async::{runtime_picker, FeagiAsyncRuntime};
    use crate::{async_number_test, async_string_formatting};

    #[test]
    fn simple_wasm_test() {
        let runtime = runtime_picker!();
        simple_async_test!(runtime);
    }
}