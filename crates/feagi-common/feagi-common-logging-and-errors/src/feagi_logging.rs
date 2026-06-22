
#[allow(unused)]
#[cfg(feature = "std")]
#[doc(hidden)]
pub use tracing as __log_tracing; // force export such that all down stream crates don't pull all of tracing

/// Allows for differentiation of a Feagi Log type
pub enum FeagiLogLevel {
    /// Information that is likely only useful for debug, and not include in release builds
    Debug,
    /// Just something to note, an event of some sort
    Info,
    /// A notice of something unexpected but automatically handled, something to be cautious of
    Warn,
    /// The failure to process some input, or something that risks a program crash
    Error,
    /// The program is crashing!
    Panic
}

#[macro_export]
macro_rules! feagi_log_debug {
    ($($arg:tt)*) => {{
        #[cfg(feature = "embedded")]
        {
            compile_error!("defmt backend for feagilog! is not implemented yet");
        }

        #[cfg(feature = "std")]
        {
            $crate::__log_tracing::debug!($($arg)*);
        }
    }};
}

#[macro_export]
macro_rules! feagi_log_info {
    ($($arg:tt)*) => {{
        #[cfg(feature = "embedded")]
        {
            compile_error!("defmt backend for feagilog! is not implemented yet");
        }

        #[cfg(feature = "std")]
        {
            $crate::__log_tracing::info!($($arg)*);
        }
    }};
}

#[macro_export]
macro_rules! feagi_log_warn {
    ($($arg:tt)*) => {{
        #[cfg(feature = "embedded")]
        {
            compile_error!("defmt backend for feagilog! is not implemented yet");
        }

        #[cfg(feature = "std")]
        {
            $crate::__log_tracing::warn!($($arg)*);
        }
    }};
}

#[macro_export]
macro_rules! feagi_log_error {
    ($($arg:tt)*) => {{
        #[cfg(feature = "embedded")]
        {
            compile_error!("defmt backend for feagilog! is not implemented yet");
        }

        #[cfg(feature = "std")]
        {
            $crate::__log_tracing::error!($($arg)*);
        }
    }};
}

#[macro_export]
macro_rules! feagi_log_panic {
    ($($arg:tt)*) => {{

        // TODO doesnt the different implementations not matter?
        /*
        #[cfg(feature = "embedded")]
        {
            compile_error!("defmt backend for feagilog! is not implemented yet");
        }

        #[cfg(feature = "std")]
        {
            panic!($($arg)*)
        }

         */
        panic!($($arg)*)
    }};
}