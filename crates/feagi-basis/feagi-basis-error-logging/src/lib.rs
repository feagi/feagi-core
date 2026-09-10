#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(feature = "enable_log_facade", feature = "enable_defmt"))]
compile_error!("features `enable_log_facade` and `enable_defmt` cannot be enabled at the same time!");

mod feagi_error;
mod feagi_logging;
extern crate self as feagi_logging_and_errors;

pub use feagi_error::{FeagiError, FeagiErrorTrait, FeagiFail, FeagiFailImpossible, FeagiFailTrait};

/// Common FEAGI error/logging imports for downstream crates.
///
/// Usage:
/// `use feagi_basis_error_logging::prelude::*;`
pub mod prelude {
    pub use crate::{
        FeagiError, FeagiErrorTrait, FeagiFail, FeagiFailImpossible, FeagiFailTrait, feagi_debug, feagi_error,
        feagi_info, feagi_warn, generate_feagi_error,
    };
}
