extern crate ndarray;
pub mod basic_components;
pub mod data;
pub mod wrapped_io_data;
pub mod genomic;
pub mod neurons;
pub mod processing;
mod templates;
mod error;

pub use templates::*;
pub use error::FeagiDataError as FeagiDataError;

