//! Binding-friendly rebuild trait.
//!
//! Bindings (PyO3, future Java) and SDK consumers implement this trait to
//! plug their concrete session type into the shared recovery loop without
//! re-implementing the decision logic.
//!
//! @cursor:critical-path
//! @cursor:ffi-safe

use crate::FeagiAgentError;

/// A FEAGI agent session that can be torn down and rebuilt in place.
///
/// Implementations must guarantee that:
/// - On `Ok(())`, the session is fully registered with FEAGI again and
///   any device registrations have been re-sent. Sensory/motor channels
///   that were active before must be active again.
/// - On `Err(_)`, no partial state remains that would prevent a future
///   `rebuild` from succeeding (transports torn down or in a re-bindable
///   state).
///
/// `reason` is forwarded to logs/telemetry; implementations should not
/// branch on its content.
pub trait RebuildableSession {
    fn rebuild(&mut self, reason: &str) -> Result<(), FeagiAgentError>;
}
