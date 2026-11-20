//! Supervisor that keeps an agent session registered.
//!
//! FEAGI ends every agent session on purpose when a genome is loaded, and a session can also be
//! lost with the link. The agent side recovers: the supervisor notices that the session is gone
//! (typed [`FeagiAgentError::NotRegistered`], a lost control socket, or a health event), asks the
//! shared [`ReconnectPolicy`] whether to re-register now, and calls the session's rebuild.
//!
//! Policy: the policy's fast attempts (`cooldown_ms` apart, `max_consecutive_failures` of them)
//! come first. When they are used up the supervisor does not give up: the state becomes
//! [`SessionRecoveryState::CannotReach`] and it keeps trying every `slow_retry_interval_ms`.
//! One session end causes one rebuild; a rebuild that succeeds clears every trigger, and a failed
//! one is spaced by the cooldown or the slow interval, so there is no tight loop.
//!
//! The supervisor never reads a clock: the driver passes `now_ms` (monotonic milliseconds).
//!
//! @cursor:critical-path
//! @cursor:cross-sdk-contract

use serde::{Deserialize, Serialize};

use crate::clients::recovery::health_watcher::{HealthEvent, HealthSnapshot, HealthWatcher};
use crate::clients::recovery::reconnect_policy::{
    ReconnectDecision, ReconnectPolicy, ReconnectPolicyConfig,
};
use crate::clients::recovery::recovery_loop::run_recovery_tick_with_events;
use crate::clients::recovery::session::RebuildableSession;
use crate::{is_transient_zmq_send_would_block, FeagiAgentError};

/// Slow retry interval used after the fast attempts are spent (product decision: about 30 s).
pub const DEFAULT_SLOW_RETRY_INTERVAL_MS: u64 = 30_000;

/// Where an agent session stands, for UIs and callers.
///
/// An agent that has never connected reports `Reconnecting` until its first registration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionRecoveryState {
    /// The session is live.
    Registered,
    /// The session ended and the fast re-registration attempts are running.
    Reconnecting,
    /// The fast attempts failed. Slow retries continue; the supervisor never stops.
    CannotReach,
}

/// Caller-supplied supervisor configuration. All of it should come from central configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SupervisorConfig {
    /// Fast-attempt policy (cooldown between attempts and how many).
    pub policy: ReconnectPolicyConfig,
    /// Interval between attempts once the fast attempts failed.
    pub slow_retry_interval_ms: u64,
    /// Upper bound for one re-registration attempt (connect + register + device registrations).
    pub rebuild_timeout_ms: u64,
}

impl SupervisorConfig {
    /// Uses [`DEFAULT_SLOW_RETRY_INTERVAL_MS`] for the slow retries.
    pub fn new(policy: ReconnectPolicyConfig, rebuild_timeout_ms: u64) -> Self {
        Self {
            policy,
            slow_retry_interval_ms: DEFAULT_SLOW_RETRY_INTERVAL_MS,
            rebuild_timeout_ms,
        }
    }
}

/// A session the supervisor can drive.
pub trait SupervisedSession {
    /// True while FEAGI knows this session.
    fn is_registered(&self) -> bool;
    /// Progress the session (heartbeats, reading FEAGI's answers). Must return
    /// [`FeagiAgentError::NotRegistered`] when the session is not live.
    fn poll_session(&mut self) -> Result<(), FeagiAgentError>;
    /// Re-register, waiting at most `timeout_ms`. Same contract as [`RebuildableSession`].
    fn rebuild_session(&mut self, reason: &str, timeout_ms: u64) -> Result<(), FeagiAgentError>;
    /// Told the supervisor's state after every tick, so the session can expose it.
    fn note_recovery_state(&mut self, _state: SessionRecoveryState) {}
}

/// What one supervised tick did.
#[derive(Debug, Clone)]
pub struct SupervisedTickReport {
    pub state: SessionRecoveryState,
    pub registered: bool,
    /// Health events observed this tick (empty when the driver gave no snapshot).
    pub health_events: Vec<HealthEvent>,
    /// True when a rebuild was invoked this tick (fast or slow).
    pub rebuild_attempted: bool,
    pub rebuild_succeeded: bool,
    pub rebuild_error: Option<String>,
    /// Milliseconds until the next slow attempt; `Some` only in `CannotReach`.
    pub next_slow_attempt_in_ms: Option<u64>,
    /// Error the session reported while being polled this tick (typed `NotRegistered` after a
    /// session end), or `None`.
    pub session_error: Option<FeagiAgentError>,
}

/// See the module documentation.
#[derive(Debug)]
pub struct SessionSupervisor {
    policy: ReconnectPolicy,
    watcher: HealthWatcher,
    slow_retry_interval_ms: u64,
    rebuild_timeout_ms: u64,
    state: SessionRecoveryState,
    next_slow_attempt_ms: Option<u64>,
}

struct Bounded<'a, S: SupervisedSession> {
    session: &'a mut S,
    timeout_ms: u64,
}

impl<S: SupervisedSession> RebuildableSession for Bounded<'_, S> {
    fn rebuild(&mut self, reason: &str) -> Result<(), FeagiAgentError> {
        self.session.rebuild_session(reason, self.timeout_ms)
    }
}

impl SessionSupervisor {
    pub fn new(config: SupervisorConfig) -> Self {
        Self {
            policy: ReconnectPolicy::new(config.policy),
            watcher: HealthWatcher::new(),
            slow_retry_interval_ms: config.slow_retry_interval_ms,
            rebuild_timeout_ms: config.rebuild_timeout_ms,
            state: SessionRecoveryState::Registered,
            next_slow_attempt_ms: None,
        }
    }

    pub fn state(&self) -> SessionRecoveryState {
        self.state
    }

    /// One tick without a health snapshot.
    pub fn tick<S: SupervisedSession>(
        &mut self,
        session: &mut S,
        now_ms: u64,
    ) -> SupervisedTickReport {
        self.tick_with_health(session, None, now_ms)
    }

    /// One tick. `health` is the driver's snapshot fetch result for this tick, or `None`.
    pub fn tick_with_health<S: SupervisedSession>(
        &mut self,
        session: &mut S,
        health: Option<Result<HealthSnapshot, FeagiAgentError>>,
        now_ms: u64,
    ) -> SupervisedTickReport {
        let session_error = session.poll_session().err();
        let transport_send_failed = session_error
            .as_ref()
            .is_some_and(|e| !e.is_not_registered() && !is_transient_zmq_send_would_block(e));
        let health_events = match health {
            Some(Ok(snapshot)) => self.watcher.observe(snapshot),
            Some(Err(_)) => self.watcher.observe_unreachable(),
            None => Vec::new(),
        };
        let registered = session.is_registered();

        // Registered again (by us, or by the caller on its own): re-arm everything.
        if registered && self.state != SessionRecoveryState::Registered {
            self.policy.reset();
            self.next_slow_attempt_ms = None;
        }

        let mut report = SupervisedTickReport {
            state: self.state,
            registered,
            health_events: health_events.clone(),
            rebuild_attempted: false,
            rebuild_succeeded: false,
            rebuild_error: None,
            next_slow_attempt_in_ms: None,
            session_error,
        };

        let tick = {
            let mut bounded = Bounded {
                session: &mut *session,
                timeout_ms: self.rebuild_timeout_ms,
            };
            run_recovery_tick_with_events(
                &mut bounded,
                &mut self.policy,
                health_events,
                transport_send_failed,
                !registered,
                now_ms,
            )
        };

        match tick.decision {
            ReconnectDecision::AttemptNow { .. } => {
                report.rebuild_attempted = true;
                report.rebuild_succeeded = tick.rebuild_succeeded;
                report.rebuild_error = tick.rebuild_error;
                self.next_slow_attempt_ms = None;
            }
            ReconnectDecision::GiveUp { .. } => {
                // Fast attempts are spent: keep trying slowly, forever.
                let due = *self.next_slow_attempt_ms.get_or_insert_with(|| {
                    self.policy
                        .last_attempt_at_ms()
                        .unwrap_or(now_ms)
                        .saturating_add(self.slow_retry_interval_ms)
                });
                self.state = SessionRecoveryState::CannotReach;
                if now_ms >= due && !session.is_registered() {
                    report.rebuild_attempted = true;
                    let mut bounded = Bounded {
                        session: &mut *session,
                        timeout_ms: self.rebuild_timeout_ms,
                    };
                    match bounded.rebuild("slow retry: client not registered") {
                        Ok(()) => {
                            report.rebuild_succeeded = true;
                            self.policy.reset();
                            self.next_slow_attempt_ms = None;
                        }
                        Err(e) => {
                            report.rebuild_error = Some(e.to_string());
                            self.next_slow_attempt_ms =
                                Some(now_ms.saturating_add(self.slow_retry_interval_ms));
                        }
                    }
                }
            }
            ReconnectDecision::Skip | ReconnectDecision::RetryAfter { .. } => {}
        }

        let registered_now = session.is_registered();
        report.registered = registered_now;
        if registered_now {
            self.state = SessionRecoveryState::Registered;
            self.next_slow_attempt_ms = None;
        } else if self.state != SessionRecoveryState::CannotReach {
            self.state = SessionRecoveryState::Reconnecting;
        }
        if self.state == SessionRecoveryState::CannotReach {
            report.next_slow_attempt_in_ms = self
                .next_slow_attempt_ms
                .map(|at| at.saturating_sub(now_ms));
        }
        report.state = self.state;
        session.note_recovery_state(self.state);
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeSession {
        registered: bool,
        server_up: bool,
        rebuild_times: Vec<u64>,
        now: u64,
        state_seen: Option<SessionRecoveryState>,
    }

    impl FakeSession {
        fn new() -> Self {
            Self {
                registered: true,
                server_up: true,
                rebuild_times: Vec::new(),
                now: 0,
                state_seen: None,
            }
        }
    }

    impl SupervisedSession for FakeSession {
        fn is_registered(&self) -> bool {
            self.registered
        }
        fn poll_session(&mut self) -> Result<(), FeagiAgentError> {
            if self.registered {
                Ok(())
            } else {
                Err(FeagiAgentError::NotRegistered("test".to_string()))
            }
        }
        fn rebuild_session(&mut self, _reason: &str, _t: u64) -> Result<(), FeagiAgentError> {
            self.rebuild_times.push(self.now);
            if self.server_up {
                self.registered = true;
                Ok(())
            } else {
                Err(FeagiAgentError::ConnectionFailed("server gone".to_string()))
            }
        }
        fn note_recovery_state(&mut self, state: SessionRecoveryState) {
            self.state_seen = Some(state);
        }
    }

    fn config() -> SupervisorConfig {
        SupervisorConfig::new(
            ReconnectPolicyConfig {
                cooldown_ms: 1_000,
                max_consecutive_failures: 3,
                trigger_on_session_changed: false,
                trigger_on_genome_changed: false,
                trigger_on_genome_load_completed: false,
                trigger_on_back_online: false,
                trigger_on_brain_ready: false,
                trigger_on_transport_send_failed: true,
            },
            5_000,
        )
    }

    fn tick(sup: &mut SessionSupervisor, s: &mut FakeSession, now: u64) -> SupervisedTickReport {
        s.now = now;
        sup.tick(s, now)
    }

    #[test]
    fn registered_session_is_left_alone() {
        let mut sup = SessionSupervisor::new(config());
        let mut s = FakeSession::new();
        for t in 0..5 {
            let r = tick(&mut sup, &mut s, t * 100);
            assert_eq!(r.state, SessionRecoveryState::Registered);
            assert!(!r.rebuild_attempted);
        }
        assert!(s.rebuild_times.is_empty());
    }

    #[test]
    fn session_end_is_recovered_once_and_the_state_follows() {
        let mut sup = SessionSupervisor::new(config());
        let mut s = FakeSession::new();
        s.registered = false; // FEAGI ended the session
        let r = tick(&mut sup, &mut s, 10);
        assert!(matches!(
            r.session_error,
            Some(FeagiAgentError::NotRegistered(_))
        ));
        assert!(r.rebuild_attempted && r.rebuild_succeeded);
        assert_eq!(r.state, SessionRecoveryState::Registered);
        assert_eq!(s.state_seen, Some(SessionRecoveryState::Registered));
        // Idempotent: no further rebuild while registered.
        for t in 1..20 {
            tick(&mut sup, &mut s, 10 + t * 10);
        }
        assert_eq!(s.rebuild_times, vec![10]);
    }

    #[test]
    fn server_gone_goes_reconnecting_then_cannot_reach_then_retries_every_30s() {
        let mut sup = SessionSupervisor::new(config());
        let mut s = FakeSession::new();
        s.registered = false;
        s.server_up = false;
        // Tick every 100 ms of mocked time for 100 s.
        let mut states = Vec::new();
        let mut t = 0u64;
        while t <= 100_000 {
            let r = tick(&mut sup, &mut s, t);
            if states.last() != Some(&r.state) {
                states.push(r.state);
            }
            t += 100;
        }
        assert_eq!(
            states,
            vec![
                SessionRecoveryState::Reconnecting,
                SessionRecoveryState::CannotReach
            ]
        );
        // Fast attempts: 3, one cooldown (1 s) apart, then slow ones 30 s apart.
        let fast = &s.rebuild_times[..3];
        assert_eq!(fast, &[0, 1_000, 2_000]);
        let slow = &s.rebuild_times[3..];
        assert!(slow.len() >= 3, "slow retries must continue: {slow:?}");
        assert_eq!(slow[0], 32_000);
        for pair in slow.windows(2) {
            assert_eq!(pair[1] - pair[0], 30_000, "{slow:?}");
        }
        // 3 fast + slow ones only: far fewer rebuilds than ticks (no tight loop).
        assert!(s.rebuild_times.len() <= 6, "{:?}", s.rebuild_times);

        // The server comes back: next slow attempt registers and the state clears.
        s.server_up = true;
        let mut recovered = None;
        while recovered.is_none() {
            let r = tick(&mut sup, &mut s, t);
            if r.registered {
                recovered = Some(r);
            }
            t += 100;
        }
        let r = recovered.unwrap();
        assert_eq!(r.state, SessionRecoveryState::Registered);
        assert_eq!(s.state_seen, Some(SessionRecoveryState::Registered));
        // And a later session end gets fast attempts again.
        s.registered = false;
        let before = s.rebuild_times.len();
        let r = tick(&mut sup, &mut s, t + 100);
        assert!(r.rebuild_attempted && r.rebuild_succeeded);
        assert_eq!(s.rebuild_times.len(), before + 1);
    }
}
