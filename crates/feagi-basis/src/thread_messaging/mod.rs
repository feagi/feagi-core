//! Structures for sending data between threads, with implementations using different backends
//! for different use cases.

pub mod errors;
pub mod data_channel_pair;
pub mod data_cycler;
