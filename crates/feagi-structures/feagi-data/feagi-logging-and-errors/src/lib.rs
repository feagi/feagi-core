
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(feature = "std", feature = "defmt"))]
compile_error!("features `std` and `defmt` cannot be enabled at the same time!");

extern crate self as feagi_logging_and_errors;

mod feagi_error;
mod feagi_logging;

pub use feagi_error::{FeagiError, FeagiErrorKey, FeagiErrorKeyTrait, FeagiErrorTrait};
pub use feagi_logging::FeagiLogType;
