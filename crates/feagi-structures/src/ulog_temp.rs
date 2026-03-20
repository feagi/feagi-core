/// Macros that log using `tracing` by default (proxy) or `defmt` when enabled.

#[doc(hidden)]
pub use tracing as __ulog_tracing; // force export such that all down stream crates don't pull all of tracing

#[macro_export]
macro_rules! udebug {
    ($($arg:tt)*) => {{
        #[cfg(feature = "defmt")]
        {
            compile_error!("defmt backend for udebug! is not implemented yet");
        }

        #[cfg(not(feature = "defmt"))]
        {
            $crate::__ulog_tracing::debug!($($arg)*);
        }
    }};
}

#[macro_export]
macro_rules! uinfo {
    ($($arg:tt)*) => {{
        #[cfg(feature = "defmt")]
        {
            compile_error!("defmt backend for uinfo! is not implemented yet");
        }

        #[cfg(not(feature = "defmt"))]
        {
            $crate::__ulog_tracing::info!($($arg)*);
        }
    }};
}

#[macro_export]
macro_rules! uwarn {
    ($($arg:tt)*) => {{
        #[cfg(feature = "defmt")]
        {
            compile_error!("defmt backend for uwarn! is not implemented yet");
        }

        #[cfg(not(feature = "defmt"))]
        {
            $crate::__ulog_tracing::warn!($($arg)*);
        }
    }};
}

#[macro_export]
macro_rules! uerror {
    ($($arg:tt)*) => {{
        #[cfg(feature = "defmt")]
        {
            compile_error!("defmt backend for uerror! is not implemented yet");
        }

        #[cfg(not(feature = "defmt"))]
        {
            $crate::__ulog_tracing::error!($($arg)*);
        }
    }};
}

#[macro_export]
macro_rules! upanic {
    ($($arg:tt)*) => {{
        #[cfg(feature = "defmt")]
        {
            compile_error!("defmt backend for upanic! is not implemented yet");
        }

        #[cfg(not(feature = "defmt"))]
        {
            panic!($($arg)*)
        }
    }};
}