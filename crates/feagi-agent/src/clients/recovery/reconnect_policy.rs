//! Reconnect decision policy.
//!
//! Pure logic. Given a stream of [`RecoveryTrigger`] inputs and the current
//! time/last-attempt state, the policy returns a [`ReconnectDecision`].
//! There are no defaults: every threshold is supplied via
//! [`ReconnectPolicyConfig`] by the caller, so the same logic applies
//! identically across the Rust SDK, the PyO3 binding, and any future Java
//! binding.
//!
//! @cursor:critical-path

use crate::clients::recovery::health_watcher::HealthEvent;
use serde::{Deserialize, Serialize};

/// Reasons a recovery attempt may be triggered.
///
/// Health-derived triggers come from [`HealthWatcher`](super::health_watcher::HealthWatcher).
/// The [`Self::TransportSendFailed`] trigger is supplied by the IO loop
/// when a publish/heartbeat operation fails in a way that is not transient
/// backpressure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryTrigger {
    Health(HealthEvent),
    TransportSendFailed,
}

/// Caller-supplied configuration for [`ReconnectPolicy`].
///
/// All fields are required and must come from centralized configuration in
/// the calling SDK / controller. The policy never substitutes defaults.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReconnectPolicyConfig {
    /// Minimum interval between reconnect attempts. Even if a new trigger
    /// arrives, the policy returns [`ReconnectDecision::RetryAfter`] until
    /// this cooldown has elapsed since the last attempt.
    pub cooldown_ms: u64,
    /// Maximum number of consecutive failed reconnect attempts before the
    /// policy returns [`ReconnectDecision::GiveUp`]. Use [`u32::MAX`] for
    /// "retry forever".
    pub max_consecutive_failures: u32,
    /// Reconnect on [`HealthEvent::SessionChanged`].
    pub trigger_on_session_changed: bool,
    /// Reconnect on [`HealthEvent::GenomeChanged`].
    pub trigger_on_genome_changed: bool,
    /// Reconnect on [`HealthEvent::GenomeLoadCompleted`].
    pub trigger_on_genome_load_completed: bool,
    /// Reconnect on [`HealthEvent::FeagiBackOnline`] (after a previous
    /// unreachable).
    pub trigger_on_back_online: bool,
    /// Reconnect on [`HealthEvent::BrainReady`] (false -> true flip).
    pub trigger_on_brain_ready: bool,
    /// Reconnect on [`RecoveryTrigger::TransportSendFailed`].
    pub trigger_on_transport_send_failed: bool,
}

/// Outcome of a single [`ReconnectPolicy::decide`] call.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReconnectDecision {
    /// No matching trigger was observed; do not reconnect.
    Skip,
    /// A trigger was observed and the cooldown has elapsed; caller should
    /// invoke its session rebuild path now.
    AttemptNow { reason: String },
    /// A trigger was observed but the cooldown has not elapsed.
    /// Caller should wait at least `wait_ms` before calling `decide` again.
    RetryAfter { wait_ms: u64, reason: String },
    /// Maximum consecutive failures reached; caller should stop retrying
    /// and surface the failure (e.g. process exit, alert).
    GiveUp { consecutive_failures: u32 },
}

/// Decision policy that maps recovery triggers to reconnect actions.
///
/// The policy keeps a small amount of state (last attempt timestamp and
/// consecutive failure count) so the same instance can be ticked from a
/// driver loop. State updates happen in [`Self::decide`],
/// [`Self::record_attempt_succeeded`], and [`Self::record_attempt_failed`].
#[derive(Debug, Clone)]
pub struct ReconnectPolicy {
    config: ReconnectPolicyConfig,
    last_attempt_at_ms: Option<u64>,
    consecutive_failures: u32,
}

impl ReconnectPolicy {
    pub fn new(config: ReconnectPolicyConfig) -> Self {
        Self {
            config,
            last_attempt_at_ms: None,
            consecutive_failures: 0,
        }
    }

    pub fn config(&self) -> &ReconnectPolicyConfig {
        &self.config
    }

    pub fn consecutive_failures(&self) -> u32 {
        self.consecutive_failures
    }

    pub fn last_attempt_at_ms(&self) -> Option<u64> {
        self.last_attempt_at_ms
    }

    /// Returns `Some(reason)` if any of the supplied triggers are configured
    /// to cause a reconnect. The reason is a stable, human-readable string
    /// that bindings can forward to logs/UI without parsing.
    fn first_matching_trigger(&self, triggers: &[RecoveryTrigger]) -> Option<String> {
        for trigger in triggers {
            match trigger {
                RecoveryTrigger::Health(event) => {
                    if self.event_is_enabled(event) {
                        return Some(format_event_reason(event));
                    }
                }
                RecoveryTrigger::TransportSendFailed => {
                    if self.config.trigger_on_transport_send_failed {
                        return Some("transport send failed".to_string());
                    }
                }
            }
        }
        None
    }

    fn event_is_enabled(&self, event: &HealthEvent) -> bool {
        match event {
            HealthEvent::SessionChanged { .. } => self.config.trigger_on_session_changed,
            HealthEvent::GenomeChanged { .. } => self.config.trigger_on_genome_changed,
            HealthEvent::GenomeLoadCompleted => self.config.trigger_on_genome_load_completed,
            HealthEvent::FeagiBackOnline => self.config.trigger_on_back_online,
            HealthEvent::BrainReady => self.config.trigger_on_brain_ready,
            HealthEvent::FeagiUnreachable
            | HealthEvent::GenomeLoadStarted
            | HealthEvent::BrainLost => false,
        }
    }

    /// Decide whether the caller should attempt to reconnect now.
    ///
    /// `now_ms` must be a monotonically non-decreasing millisecond clock
    /// supplied by the driver. The policy itself does not read time.
    pub fn decide(&mut self, triggers: &[RecoveryTrigger], now_ms: u64) -> ReconnectDecision {
        if self.consecutive_failures >= self.config.max_consecutive_failures
            && self.config.max_consecutive_failures != u32::MAX
        {
            return ReconnectDecision::GiveUp {
                consecutive_failures: self.consecutive_failures,
            };
        }

        let Some(reason) = self.first_matching_trigger(triggers) else {
            return ReconnectDecision::Skip;
        };

        if let Some(last) = self.last_attempt_at_ms {
            let elapsed = now_ms.saturating_sub(last);
            if elapsed < self.config.cooldown_ms {
                return ReconnectDecision::RetryAfter {
                    wait_ms: self.config.cooldown_ms - elapsed,
                    reason,
                };
            }
        }

        self.last_attempt_at_ms = Some(now_ms);
        ReconnectDecision::AttemptNow { reason }
    }

    /// Reset failure counter after a successful rebuild.
    pub fn record_attempt_succeeded(&mut self) {
        self.consecutive_failures = 0;
    }

    /// Increment failure counter after a failed rebuild.
    pub fn record_attempt_failed(&mut self) {
        self.consecutive_failures = self.consecutive_failures.saturating_add(1);
    }

    /// Forget any prior attempt state. Useful when a SDK manually
    /// reconnects outside the policy and wants to re-arm the cooldown.
    pub fn reset(&mut self) {
        self.last_attempt_at_ms = None;
        self.consecutive_failures = 0;
    }
}

fn format_event_reason(event: &HealthEvent) -> String {
    match event {
        HealthEvent::SessionChanged { old, new } => {
            format!("feagi session changed ({} -> {})", old, new)
        }
        HealthEvent::GenomeChanged {
            old_num,
            new_num,
            old_timestamp,
            new_timestamp,
        } => format!(
            "genome changed (num: {:?} -> {:?}, ts: {:?} -> {:?})",
            old_num, new_num, old_timestamp, new_timestamp
        ),
        HealthEvent::GenomeLoadCompleted => "genome load completed".to_string(),
        HealthEvent::FeagiBackOnline => "feagi back online".to_string(),
        HealthEvent::BrainReady => "brain ready".to_string(),
        HealthEvent::FeagiUnreachable => "feagi unreachable".to_string(),
        HealthEvent::GenomeLoadStarted => "genome load started".to_string(),
        HealthEvent::BrainLost => "brain lost".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> ReconnectPolicyConfig {
        ReconnectPolicyConfig {
            cooldown_ms: 1_000,
            max_consecutive_failures: 5,
            trigger_on_session_changed: true,
            trigger_on_genome_changed: true,
            trigger_on_genome_load_completed: true,
            trigger_on_back_online: true,
            trigger_on_brain_ready: false,
            trigger_on_transport_send_failed: true,
        }
    }

    #[test]
    fn no_triggers_skip() {
        let mut p = ReconnectPolicy::new(cfg());
        let decision = p.decide(&[], 0);
        assert_eq!(decision, ReconnectDecision::Skip);
    }

    #[test]
    fn disabled_event_does_not_trigger() {
        let mut p = ReconnectPolicy::new(cfg());
        let decision = p.decide(&[RecoveryTrigger::Health(HealthEvent::BrainReady)], 0);
        assert_eq!(decision, ReconnectDecision::Skip);
    }

    #[test]
    fn first_matching_trigger_attempts_now() {
        let mut p = ReconnectPolicy::new(cfg());
        let decision = p.decide(
            &[RecoveryTrigger::Health(HealthEvent::GenomeLoadCompleted)],
            0,
        );
        match decision {
            ReconnectDecision::AttemptNow { reason } => {
                assert_eq!(reason, "genome load completed");
            }
            other => panic!("expected AttemptNow, got {:?}", other),
        }
    }

    #[test]
    fn cooldown_blocks_immediate_retry() {
        let mut p = ReconnectPolicy::new(cfg());
        let _ = p.decide(
            &[RecoveryTrigger::Health(HealthEvent::GenomeLoadCompleted)],
            0,
        );
        let decision = p.decide(
            &[RecoveryTrigger::Health(HealthEvent::GenomeLoadCompleted)],
            500,
        );
        match decision {
            ReconnectDecision::RetryAfter { wait_ms, .. } => assert_eq!(wait_ms, 500),
            other => panic!("expected RetryAfter, got {:?}", other),
        }
    }

    #[test]
    fn cooldown_elapsed_allows_retry() {
        let mut p = ReconnectPolicy::new(cfg());
        let _ = p.decide(&[RecoveryTrigger::TransportSendFailed], 0);
        let decision = p.decide(&[RecoveryTrigger::TransportSendFailed], 1_001);
        assert!(matches!(decision, ReconnectDecision::AttemptNow { .. }));
    }

    #[test]
    fn max_failures_gives_up() {
        let mut p = ReconnectPolicy::new(cfg());
        for _ in 0..5 {
            p.record_attempt_failed();
        }
        let decision = p.decide(
            &[RecoveryTrigger::Health(HealthEvent::GenomeLoadCompleted)],
            10_000,
        );
        match decision {
            ReconnectDecision::GiveUp {
                consecutive_failures,
            } => assert_eq!(consecutive_failures, 5),
            other => panic!("expected GiveUp, got {:?}", other),
        }
    }

    #[test]
    fn record_success_resets_failure_counter() {
        let mut p = ReconnectPolicy::new(cfg());
        p.record_attempt_failed();
        p.record_attempt_failed();
        p.record_attempt_succeeded();
        assert_eq!(p.consecutive_failures(), 0);
    }

    #[test]
    fn u32_max_failures_means_retry_forever() {
        let mut config = cfg();
        config.max_consecutive_failures = u32::MAX;
        let mut p = ReconnectPolicy::new(config);
        for _ in 0..1_000 {
            p.record_attempt_failed();
        }
        let decision = p.decide(&[RecoveryTrigger::TransportSendFailed], 10_000);
        assert!(matches!(decision, ReconnectDecision::AttemptNow { .. }));
    }

    #[test]
    fn transport_send_failed_can_be_disabled() {
        let mut config = cfg();
        config.trigger_on_transport_send_failed = false;
        let mut p = ReconnectPolicy::new(config);
        let decision = p.decide(&[RecoveryTrigger::TransportSendFailed], 0);
        assert_eq!(decision, ReconnectDecision::Skip);
    }
}
