
#![cfg_attr(not(feature = "std"), no_std)]


extern crate self as feagi_logging_and_errors;

mod feagi_error;
mod feagi_logging;

pub use feagi_error::{FeagiError, FeagiErrorKey, FeagiErrorKeyTrait, FeagiErrorTrait};
