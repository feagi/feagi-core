//! This library Contains the core common systems used throughout Feagi

mod core_numerical_types;
mod linear_index_types;
mod feagi_data_error;


pub mod percentages;
pub mod quantizable;
pub mod bit_packing;
pub mod feagi_error;
pub mod feagi_ecs;
pub mod collection_traits;
pub mod spatial;


pub use linear_index_types::LinearIndexCountType;
pub use core_numerical_types::{SupportsBasicCoreMathOps, SupportsUintOps};
pub use feagi_data_error::FeagiDataError;
pub use paste;