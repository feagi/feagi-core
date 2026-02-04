//! Perception decoder for motor outputs.

mod config;
mod decoder;

pub use config::PerceptionDecoderConfig;
pub use decoder::{OimgFrame, OsegFrame, PerceptionDecoder, PerceptionFrame};
