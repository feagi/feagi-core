//! This crate defines the basic ECS traits used by FEAGI
// TODO we are using a subcrate since we will be adding some derive macros under this!

extern crate self as feagi_ecs;

pub mod device;
pub mod memory;
pub mod metadata;