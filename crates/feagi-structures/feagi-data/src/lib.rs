//! This library contains Quantization Traits, Types, and Macros to generate those types easily
//! It also contains bit packing systems

// only contains macros, no need to export

mod linear_index_types;
mod feagi_data_error;

pub mod percentages;
pub mod quantizable;
pub mod bit_packing;
pub mod feagi_error;
pub mod feagi_ecs;
pub mod linear_collections;
pub mod spatial;

pub use linear_index_types::LinearIndexCountType;
pub use feagi_data_error::FeagiDataError;
pub use paste;