//! The Feagi Parallel Interface crate is a loose definition of traits that help group
//! elements


extern crate self as feagi_pdi;

mod main_traits;


pub mod intercollection_data_flow;
pub mod tag_device;

pub use main_traits::*;

// TODO can we make some sort of trait for System?