//! Cortical Models

pub mod foundation_power;
pub mod foundation_death;
pub mod feagi_advanced;
pub mod memory_fire_hashing;
pub mod feagi_simple;

#[macro_use]
pub mod family;
pub mod neurogenesis_dispatch;
pub mod defaults_generator;
// Replaced by neurogenesis, as the `NeuronInitializer` trait they implement no longer exists
//pub mod initializers;

