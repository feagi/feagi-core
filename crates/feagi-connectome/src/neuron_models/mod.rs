//! Cortical Models

pub mod foundation_power;
pub mod foundation_death;
pub mod feagi_advanced;
pub mod memory_fire_hashing;
pub mod feagi_simple;

pub use family::neuron_model;

#[macro_use]
pub mod family;
pub mod spec_dispatch;

