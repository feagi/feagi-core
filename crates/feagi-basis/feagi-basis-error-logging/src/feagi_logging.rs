

#[macro_export]
macro_rules! feagi_debug {
    ($($arg:tt)*) => {{
        #[cfg(feature = "enable_defmt")]
        {
            compile_error!("defmt backend for feagilog! is not implemented yet");
        }

        #[cfg(feature = "enable_log_facade")]
        {
            $crate::__log_tracing::debug!($($arg)*);
        }
    }};
}

#[macro_export]
macro_rules! feagi_info {
    ($($arg:tt)*) => {{
        #[cfg(feature = "enable_defmt")]
        {
            compile_error!("defmt backend for feagilog! is not implemented yet");
        }

        #[cfg(feature = "enable_log_facade")]
        {
            $crate::__log_tracing::info!($($arg)*);
        }
    }};
}

#[macro_export]
macro_rules! feagi_warn {
    ($($arg:tt)*) => {{
        #[cfg(feature = "enable_defmt")]
        {
            compile_error!("defmt backend for feagilog! is not implemented yet");
        }

        #[cfg(feature = "enable_log_facade")]
        {
            $crate::__log_tracing::warn!($($arg)*);
        }
    }};
}

#[macro_export]
macro_rules! feagi_error {
    ($($arg:tt)*) => {{
        #[cfg(feature = "enable_defmt")]
        {
            compile_error!("defmt backend for feagilog! is not implemented yet");
        }

        #[cfg(feature = "enable_log_facade")]
        {
            $crate::__log_tracing::error!($($arg)*);
        }
    }};
}
