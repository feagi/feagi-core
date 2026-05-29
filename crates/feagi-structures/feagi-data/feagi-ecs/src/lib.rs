//! This crate defines the basic ECS traits used by FEAGI
//! Inm our context, ECS type data could live on various devices (CPU RAM, WGPU VRAM, etc) and thus
//! we need a general interface and tagging to handle this
// TODO we are using a subcrate since we may be adding some derive macros under this!

extern crate self as feagi_ecs;

pub mod element; // E for Element, though rust devs often already have e
pub mod element_set;
pub mod collection; // C for collection
pub mod intercollection_data_flow;
pub mod tag_device;
// TODO can we make some sort of trait for System?