//! Built-in sampler implementations.

pub mod sequential;
pub mod shuffled;

pub use sequential::SequentialSampler;
pub use shuffled::ShuffledSampler;
