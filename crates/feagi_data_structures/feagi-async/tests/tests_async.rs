//! Tests for the various async implementations

async fn async_number_test(x: i32, y: i32) -> i32 {
    x + y
}

async fn async_string_formatting(name: String) -> String {
    format!("Hello, {}!", name)
}


#[cfg(test)]
#[cfg(feature = "standard-tokio")]
mod tests_tokio {
    use feagi_async::runtime_picker;

    #[test]
    fn simple_tokio_test() {
        let runtime = runtime_picker!();
    }







}