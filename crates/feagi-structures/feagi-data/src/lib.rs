//! This library Contains the core common systems used throughout Feagi

// Reexpose subcrates
pub use feagi_ecs;
pub use feagi_logging_and_errors;

mod core_numerical_types;


pub mod percentages;
pub mod quantizable;
pub mod bit_packing;
pub mod spatial;

pub use core_numerical_types::{SupportsBasicCoreMathOps, SupportsUintOps};
pub use paste;