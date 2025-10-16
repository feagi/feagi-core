//! Caching infrastructure for FEAGI I/O operations.
//!
//! Provides high-performance caching for sensor inputs and motor outputs,
//! with automatic encoding/decoding to/from neuron voxel representations.
//! The cache handles data preprocessing, pipeline management, and type conversions.

mod io_motor_cache;
mod io_sensor_cache;
mod io_cache;
pub(crate) mod per_channel_stream_caches;

pub use io_cache::IOCache;