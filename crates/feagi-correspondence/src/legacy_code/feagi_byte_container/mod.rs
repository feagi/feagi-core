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
//!
//! More information about the specification can be found in the documentation.

// TODO this crate needs to be deprecated


mod feagi_byte_container;
mod feagi_byte_structure_type;
mod feagi_json;
mod feagi_serializable;
pub mod implementations;

pub use feagi_byte_container::{AgentIdentifier, FeagiByteContainer};
pub use feagi_byte_structure_type::FeagiByteStructureType;
pub use feagi_json::FeagiJSON;
pub use feagi_serializable::FeagiSerializable;
