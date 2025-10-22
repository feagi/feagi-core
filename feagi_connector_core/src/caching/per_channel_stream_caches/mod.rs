//! Per-channel stream caching for sensor and motor data.
//!
//! Provides independent caching and pipeline processing for each channel
//! within sensor and motor cortical groups.

mod sensory_channel_stream_caches;
mod motor_channel_stream_caches;

pub(crate) use sensory_channel_stream_caches::SensoryChannelStreamCaches;
pub(crate) use motor_channel_stream_caches::MotorChannelStreamCaches;

