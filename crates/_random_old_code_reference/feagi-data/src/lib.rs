//! This library Contains the core common systems used throughout Feagi

extern crate self as feagi_data;
//extern crate feagi_logging_and_errors;

// Reexpose subcrates
pub use feagi_logging_and_errors;
pub use feagi_pdi;
pub use feagi_bitpacking;

mod core_numerical_types;


pub mod percentages;
pub mod quantizable_linear;
pub mod quantizable_spatial;
pub mod quantizable_collections;
pub mod common_const_labels;
pub mod quantization_levels;

pub use core_numerical_types::{SupportsBasicCoreMathOps, SupportsUintOps};
pub use paste;