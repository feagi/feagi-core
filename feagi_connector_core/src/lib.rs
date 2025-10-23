//! # FEAGI Connector Core
//!
//! Core functionality for FEAGI connectors including caching, data processing,
//! and neuron voxel encoding/decoding.
//!
//! This crate provides the essential infrastructure for connecting sensors and
//! actuators to FEAGI's neural processing engine, handling data transformation
//! between various formats and neuron voxel representations.
//!
//! ## Main Components
//!
//! - **[`IOCache`]** - High-level caching system for sensor inputs and motor outputs
//! - **[`data_types`]** - Data structures for images, percentages, and misc sensor/motor data
//! - **[`data_pipeline`]** - Pipeline stages for preprocessing/postprocessing data
//! - **[`wrapped_io_data`]** - Type-safe wrappers for heterogeneous I/O data
//!
//! ## Quick Start
//!
//! ```rust
//! use feagi_connector_core::IOCache;
//! use feagi_data_structures::genomic::descriptors::CorticalGroupIndex;
//!
//! // Create a new I/O cache
//! let mut cache = IOCache::new();
//!
//! // Cache manages sensor inputs and motor outputs with automatic
//! // encoding/decoding to/from neuron voxel representations
//! ```
//!
//! ## Architecture
//!
//! Data flows through the connector in this sequence:
//!
//! 1. **Raw Input** (images, sensor values, etc.)
//! 2. **Pre-processing Pipeline** (optional transformations)
//! 3. **Neuron Encoding** (convert to voxel representations)
//! 4. **FEAGI Processing** (neural network computation)
//! 5. **Neuron Decoding** (convert from voxels to values)
//! 6. **Post-processing Pipeline** (optional transformations)
//! 7. **Motor Output** (actuator commands)

mod caching;
mod neuron_voxel_coding;

pub mod data_pipeline;
pub mod wrapped_io_data;
pub mod data_types;
mod connector_agent;
mod sensor_device_cache;
mod motor_device_cache;

pub use caching::IOCache;
