#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(feature = "enable_log_facade", feature = "enable_defmt"))]
compile_error!("features `enable_log_facade` and `enable_defmt` cannot be enabled at the same time!");

pub mod feagi_error;
pub mod feagi_logging;
pub mod request_response_traits;


/// Common FEAGI imports for downstream crates.
pub mod prelude {
    pub use crate::{
        feagi_debug, feagi_info, feagi_warn, generate_feagi_error, // logging
        feagi_error::FeagiFail, feagi_error::FeagiFailTrait, feagi_error::FeagiFailImpossible, // Feagi Fail
        feagi_error::FeagiError, feagi_error::FeagiErrorTrait, // feagi error
    };
    pub use feagi_macros_proc::bit_struct_builder;

}

#[cfg(feature = "__feagi_request_response_macros")]
pub mod request_responses {
    pub use feagi_macros_proc::template_request_category;
    pub use feagi_macros_proc::generate_from_template_request_response_structs_and_enums;

    #[cfg(feature = "feagi_server_request_rest")]
    pub use feagi_macros_proc::generate_from_template_ohkami_rest_server;
    
}