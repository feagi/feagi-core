#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(feature = "enable_log_facade", feature = "enable_defmt"))]
compile_error!("features `enable_log_facade` and `enable_defmt` cannot be enabled at the same time!");

pub mod feagi_error;
pub mod feagi_logging;

/// Common FEAGI error/logging imports for downstream crates.
///
/// Usage:
/// `use feagi_basis_error_logging::prelude::*;`
pub mod prelude {
    pub use crate::{
        feagi_debug, feagi_error, feagi_error::FeagiError, feagi_error::FeagiErrorTrait, 
        feagi_error::FeagiFail, feagi_error::FeagiFailImpossible,
        feagi_error::FeagiFailTrait, feagi_info, feagi_warn, generate_feagi_error,
    };
}
