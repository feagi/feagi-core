//! ZMQ Transport Module
//!
//! Provides ZMQ-based control plane for the FEAGI API using feagi-transports.

pub mod adapter;

pub use adapter::ZmqApiAdapter;
