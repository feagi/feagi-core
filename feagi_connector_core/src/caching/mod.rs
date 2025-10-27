//! Caching infrastructure for FEAGI I/O operations.
//!
//! Provides high-performance caching for sensor inputs and motor outputs,
//! with automatic encoding/decoding to/from neuron voxel representations.
//! The cache handles data preprocessing, pipeline management, and type conversions.
pub(crate) mod per_channel_stream_caches;
