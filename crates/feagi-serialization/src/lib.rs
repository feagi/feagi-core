//! # FEAGI Data Serialization
//!
//! This crate provides traits and utilities for serializing and deserializing various data structures
//! to and from byte vectors in the FEAGI system. It offers a unified serialization framework through
//! the [`FeagiSerializable`] trait and efficient byte data management via [`FeagiByteContainer`].
//!
//! ## Core Components
//!
//! - **[`FeagiSerializable`]** - Common trait for structures that can be serialized to/from bytes
//! - **[`FeagiByteContainer`]** - Container that manages and owns byte data for multiple structures
//! - **[`FeagiByteStructureType`]** - Enum identifying different serializable structure types
//!
//!
//! ## Basic Usage
//!
//! ```rust
//! use feagi_serialization::{FeagiByteContainer, FeagiSerializable};
//!
//! // Create an empty container
//! let mut container = FeagiByteContainer::new_empty();
//! assert!(container.is_valid());
//! assert_eq!(container.try_get_number_contained_structures().unwrap(), 0);
//!
//! // Get information about the container
//! let byte_count = container.get_number_of_bytes_used();
//! let struct_types = container.get_contained_struct_types();
//! ```
//!
//! More information about the specification can be found in the documentation.

mod feagi_byte_container;
mod feagi_byte_structure_type;
mod feagi_serializable;
pub mod implementations;

pub use feagi_byte_container::FeagiByteContainer;
pub use feagi_byte_structure_type::FeagiByteStructureType;
pub use feagi_serializable::FeagiSerializable;
