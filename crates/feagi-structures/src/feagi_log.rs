/// Macros that log using `tracing` by default (proxy) or `defmt` when enabled.

// TODO WASM?

#[doc(hidden)]
pub use tracing as __log_tracing; // force export such that all down stream crates don't pull all of tracing

#[macro_export]
macro_rules! feagi_log_debug {
    ($($arg:tt)*) => {{
        #[cfg(feature = "defmt")]
        {
            compile_error!("defmt backend for feagilog! is not implemented yet");
        }

        #[cfg(not(feature = "defmt"))]
        {
            $crate::__log_tracing::debug!($($arg)*);
        }
    }};
}

#[macro_export]
macro_rules! feagi_log_info {
    ($($arg:tt)*) => {{
        #[cfg(feature = "defmt")]
        {
            compile_error!("defmt backend for feagilog! is not implemented yet");
        }

        #[cfg(not(feature = "defmt"))]
        {
            $crate::__log_tracing::info!($($arg)*);
        }
    }};
}

#[macro_export]
macro_rules! feagi_log_warn {
    ($($arg:tt)*) => {{
        #[cfg(feature = "defmt")]
        {
            compile_error!("defmt backend for feagilog! is not implemented yet");
        }

        #[cfg(not(feature = "defmt"))]
        {
            $crate::__log_tracing::warn!($($arg)*);
        }
    }};
}

#[macro_export]
macro_rules! feagi_log_error {
    ($($arg:tt)*) => {{
        #[cfg(feature = "defmt")]
        {
            compile_error!("defmt backend for feagilog! is not implemented yet");
        }

        #[cfg(not(feature = "defmt"))]
        {
            $crate::__log_tracing::error!($($arg)*);
        }
    }};
}

#[macro_export]
macro_rules! feagi_log_panic {
    ($($arg:tt)*) => {{
        #[cfg(feature = "defmt")]
        {
            compile_error!("defmt backend for feagilog! is not implemented yet");
        }

        #[cfg(not(feature = "defmt"))]
        {
            panic!($($arg)*)
        }
    }};
}