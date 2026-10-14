//! Two-stage Zenoh session configuration.
//!
//! [`configuration_builder::ZenohConfigurationBuilder`] holds settings that do not depend on node mode.
//! Choosing client, peer, or router produces a [`node::ZenohNodeBuilder`] whose connect and
//! listen values are for that one mode. [`node::ZenohNodeBuilder::build_json5`] writes the document.

pub mod configuration_builder;
pub mod connect;
pub mod error;
pub mod id_generation_mode;
pub mod namespace;
pub mod node;
pub mod node_mode;
pub mod queries_default_timeout;
pub mod transport;
