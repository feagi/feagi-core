//! This crate defines the basic ECS traits used by FEAGI
// TODO we are using a subcrate since we will be adding some derive macros under this!

extern crate self as feagi_ecs;

pub mod element; // E for Element, though rust devs often already have e
pub mod collection; // C for collection
pub mod intercollection_data_flow;

// TODO can we make some sort of trait for System?