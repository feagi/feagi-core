//! Video encoder implementation.

mod config;
mod encoder;

pub use config::{VideoEncoderConfig, VideoEncodingStrategy};
pub use encoder::VideoEncoder;
