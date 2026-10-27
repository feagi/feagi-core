//! Latest value signals. Notifiers push a value, and a single listener takes the newest one,
//! either by checking or by awaiting. Values not yet taken are replaced (or merged) rather than
//! queued, so this suits state changes and "something happened" notices, not data streams. Use
//! feagi channels when every value must arrive
//!
//! Notifiers are clonable and never block. Once every notifier is dropped the signal is closed,
//! which the listener sees after taking any value still pending

pub mod signal_traits;
pub mod signal_errors;
pub mod implementations;

mod signal_state;
