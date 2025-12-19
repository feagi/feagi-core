mod embedded_esp_32;
#[cfg(feature = "async-tokio")]
mod standard_tokio;
mod wasi;
#[cfg(feature = "async-wasm")]
mod wasm;

#[cfg(feature = "async-tokio")]
pub use standard_tokio::{TokioHandle, TokioRuntime};

#[cfg(feature = "async-wasm")]
pub use wasm::WasmRuntime;
