//! Integration test: a balance phase against a fake controller control server over real HTTP.
//!
//! The fake server speaks the controller contract the Trainer relies on: `POST /command` with
//! an `action`, the `{ok, result}` envelope, `link: {state, epoch}` on `reset_episode`,
//! `measure` and `status`, and HTTP 503 with `retryable: true` when its physics loop is busy.
//! It knows no robot: the model is a few generically named bodies.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use feagi_trainer::balance_feet::{
    run_balance_phase, BalanceFeetCriterion, BalancePhaseRun, BalanceRewardPolicy,
    HttpBalanceController,
};
use feagi_trainer::balance_recovery::{BurstClock, DiscardReason, LinkState, RecoveryConfig};
use feagi_trainer::contracts::RunId;
use feagi_trainer::control::{CancelToken, CollectingEventSink};
use feagi_trainer::error::TrainerError;
use serde_json::{json, Value};

/// Seconds simulated per burst.
const BURST_S: f64 = 0.05;
/// Physics step of the fake model, seconds.
const STEP_S: f64 = 0.002;

/// What the fake controller does during the run.
#[derive(Clone, Copy)]
struct Scenario {
    /// The link drops (reconnecting, epoch + 1) when this many `measure` calls were served.
    outage_at_measure: Option<u64>,
    /// How long the link stays down before it is `linked` again (epoch + 1 once more).
    outage: Duration,
    /// This `measure` call (0-based count) is answered 503 `retryable: true`, once.
    busy_at_measure: Option<u64>,
    /// `reset_episode` replies carry no `link` (a controller that predates the contract).
    omit_link_on_reset: bool,
}

/// FEAGI's burst engine is outside the subject: a clock that is always running.
struct RunningClock(std::sync::atomic::AtomicU64);

impl BurstClock for RunningClock {
    fn burst_count(&self) -> Result<u64, TrainerError> {
        Ok(self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1)
    }

    fn engine_paused(&self) -> Result<bool, TrainerError> {
        Ok(false)
    }
}

struct State {
    scenario: Scenario,
    episode: u64,
    k: u64,
    fresh: bool,
    measures: u64,
    resets: u32,
    statuses: u32,
    link_state: &'static str,
    link_epoch: u64,
    outage_until: Option<Instant>,
    busy_fired: bool,
}

impl State {
    /// The link as the controller reports it now; an outage that has run its course ends here.
    fn link(&mut self) -> Value {
        if self.link_state == "reconnecting"
            && self.outage_until.is_some_and(|t| Instant::now() >= t)
        {
            self.link_state = "linked";
            self.link_epoch += 1;
            self.outage_until = None;
        }
        json!({"state": self.link_state, "epoch": self.link_epoch})
    }
}

fn model_info() -> Value {
    let body = |name: &str, mass: f64| {
        json!({
            "name": name, "root_name": "base", "mass": mass, "subtree_mass": mass,
            "inertia": [2e-3, 2e-3, 1e-3], "subtree_inertia": [5e-3, 4e-3, 1.3e-3]
        })
    };
    json!({
        "timestep_s": STEP_S, "gravity": [0.0, 0.0, -9.81],
        "bodies": [body("base", 1.0), body("torso", 0.6), body("contact_a", 0.05), body("contact_b", 0.05), body("head", 0.1)],
        "free_roots": ["base"], "keyframes": ["home"]
    })
}

/// Answer to one request: `(status, body)`.
fn answer(state: &mut State, request: &Value) -> (u16, Value) {
    let ok = |result: Value| (200, json!({"ok": true, "result": result}));
    match request["action"].as_str().unwrap_or("") {
        "get_model_info" => ok(model_info()),
        "reset_episode" => {
            state.episode += 1;
            state.resets += 1;
            state.k = 0;
            state.fresh = true;
            if state.scenario.omit_link_on_reset {
                return ok(json!({"episode_id": state.episode}));
            }
            ok(json!({"episode_id": state.episode, "link": state.link()}))
        }
        "status" => {
            state.statuses += 1;
            ok(json!({"link": state.link()}))
        }
        "measure" => {
            let index = state.measures;
            state.measures += 1;
            if state.scenario.busy_at_measure == Some(index) && !state.busy_fired {
                state.busy_fired = true;
                return (
                    503,
                    json!({"ok": false, "retryable": true, "error": "physics loop busy"}),
                );
            }
            if state.scenario.outage_at_measure == Some(index) {
                state.link_state = "reconnecting";
                state.link_epoch += 1;
                state.outage_until = Some(Instant::now() + state.scenario.outage);
            }
            if state.fresh {
                state.fresh = false;
            } else {
                state.k += 1;
            }
            let contact = json!({"touching": true, "points": [[0.0, 0.0, 0.01]]});
            ok(json!({
                "sim_time_s": state.k as f64 * BURST_S,
                "physics_steps": (state.k as f64 * BURST_S / STEP_S) as u64,
                "episode_id": state.episode,
                "bodies": {"torso": {
                    "tilt_rad": 0.03, "ang_vel": [0.0, 0.0, 0.0],
                    "com": [0.0, 0.0, 0.1], "subtree_com": [0.0, 0.0, 0.15]
                },
                "contact_a": {"tilt_rad": 0.0, "com": [0.0, 0.05, 0.0]},
                "contact_b": {"tilt_rad": 0.0, "com": [0.0, -0.05, 0.0]},
                "head": {"tilt_rad": 0.0, "com": [0.0, 0.0, 0.3]}},
                "contacts": [contact.clone(), contact],
                "springs": {},
                "link": state.link()
            }))
        }
        // Spring, pulse and any other action: accepted, nothing to report.
        _ => ok(json!({})),
    }
}

/// Starts the fake control server; returns its port and its shared state.
fn serve(scenario: Scenario) -> (u16, Arc<Mutex<State>>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().expect("addr").port();
    let state = Arc::new(Mutex::new(State {
        scenario,
        episode: 0,
        k: 0,
        fresh: false,
        measures: 0,
        resets: 0,
        statuses: 0,
        link_state: "linked",
        link_epoch: 1,
        outage_until: None,
        busy_fired: false,
    }));
    let shared = state.clone();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(stream) = stream else { return };
            let state = shared.clone();
            std::thread::spawn(move || {
                let mut reader = BufReader::new(stream);
                loop {
                    let mut length = 0usize;
                    loop {
                        let mut line = String::new();
                        match reader.read_line(&mut line) {
                            Ok(0) | Err(_) => return,
                            Ok(_) => {}
                        }
                        let lower = line.trim_end().to_ascii_lowercase();
                        if let Some(value) = lower.strip_prefix("content-length:") {
                            length = value.trim().parse().unwrap_or(0);
                        }
                        if lower.is_empty() {
                            break;
                        }
                    }
                    let mut body = vec![0u8; length];
                    if reader.read_exact(&mut body).is_err() {
                        return;
                    }
                    let request: Value = serde_json::from_slice(&body).unwrap_or(Value::Null);
                    let (status, payload) = answer(&mut state.lock().expect("state"), &request);
                    let payload = payload.to_string();
                    let head = format!(
                        "HTTP/1.1 {status} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: keep-alive\r\n\r\n",
                        payload.len()
                    );
                    let socket = reader.get_mut();
                    if socket.write_all(head.as_bytes()).is_err()
                        || socket.write_all(payload.as_bytes()).is_err()
                    {
                        return;
                    }
                }
            });
        }
    });
    (port, state)
}

/// An attempt succeeds after 10 upright bursts (hold 0.5 s).
fn criterion() -> BalanceFeetCriterion {
    let mut c = BalanceFeetCriterion::new(
        "torso",
        vec!["contact_a".to_string(), "contact_b".to_string()],
        "head",
        15.0,
        0.5,
        2.0,
    );
    c.burst_period_s = BURST_S;
    c.settle_s = 0.0;
    c.probe_every = 0;
    c.only_contacts_touch = false;
    c
}

fn run_phase(
    port: u16,
    seeds: &[u64],
    recovery: &RecoveryConfig,
) -> Result<feagi_trainer::balance_feet::BalancePhaseReport, TrainerError> {
    let mut controller =
        HttpBalanceController::new(&format!("http://127.0.0.1:{port}"), Duration::from_secs(5))?;
    let c = criterion();
    let run_id = RunId("recovery-integration".to_string());
    let mut run = BalancePhaseRun {
        phase: "train",
        seeds,
        criterion: &c,
        run_id: &run_id,
        support_mode: None,
        deliver_reward: None,
        plasticity: None,
        reward_policy: BalanceRewardPolicy::Affect,
        data: None,
        apply_event: None,
        recovery,
        burst: &RunningClock(std::sync::atomic::AtomicU64::new(0)),
        memory_counters: None,
    };
    run_balance_phase(
        &mut run,
        &mut controller,
        &mut |_| Ok(()),
        &mut CollectingEventSink::default(),
        &CancelToken::new(),
    )
}

#[test]
fn a_link_outage_mid_attempt_discards_it_and_the_phase_finishes() {
    // The link drops during the second attempt (measure 15 of 11 per attempt) for 0.4 s.
    let (port, state) = serve(Scenario {
        outage_at_measure: Some(15),
        outage: Duration::from_millis(400),
        busy_at_measure: None,
        omit_link_on_reset: false,
    });
    let recovery = RecoveryConfig::new(5.0, 3, 0.02, 0.02, 1.0).expect("recovery");
    let report = run_phase(port, &[1, 2, 3], &recovery).expect("phase finishes");

    // Three scored attempts, one per seed, in order; the outage left no score behind.
    let seeds: Vec<u64> = report.attempts.iter().map(|a| a.seed).collect();
    assert_eq!(seeds, [1, 2, 3]);
    assert_eq!(report.success_rate, 1.0);
    assert!(report.attempts.iter().all(|a| a.bursts == 10));
    // Exactly one recorded discard: seed 2, link went reconnecting at epoch 2.
    assert_eq!(report.discarded_attempts.len(), 1);
    let discard = &report.discarded_attempts[0];
    assert_eq!((discard.seed, discard.attempt_index), (2, 1));
    assert_eq!(discard.reason, DiscardReason::LinkNotLinked);
    assert_eq!(discard.link_state_after, Some(LinkState::Reconnecting));
    assert_eq!(
        (discard.link_epoch_before, discard.link_epoch_after),
        (Some(1), Some(2))
    );
    // The discarded seed was reset again after the link returned (epoch 3): 3 seeds + 1 re-run.
    let state = state.lock().expect("state");
    assert_eq!(state.resets, 4);
    assert!(
        state.statuses >= 2,
        "the wait polled status: {}",
        state.statuses
    );
    assert_eq!(state.link_epoch, 3);
}

#[test]
fn an_outage_that_outlasts_max_outage_fails_the_phase_naming_the_limit() {
    let (port, _state) = serve(Scenario {
        outage_at_measure: Some(15),
        outage: Duration::from_secs(30),
        busy_at_measure: None,
        omit_link_on_reset: false,
    });
    let recovery = RecoveryConfig::new(0.2, 3, 0.02, 0.02, 1.0).expect("recovery");
    let error = run_phase(port, &[1, 2, 3], &recovery).unwrap_err();
    let message = error.to_string();
    assert!(
        matches!(error, TrainerError::Runtime(_)) && message.contains("max_outage_s (0.2)"),
        "{message}"
    );
}

#[test]
fn a_busy_physics_loop_is_ridden_out_as_one_discard() {
    let (port, state) = serve(Scenario {
        outage_at_measure: None,
        outage: Duration::ZERO,
        busy_at_measure: Some(5),
        omit_link_on_reset: false,
    });
    let recovery = RecoveryConfig::new(5.0, 3, 0.02, 0.02, 1.0).expect("recovery");
    let report = run_phase(port, &[1, 2], &recovery).expect("phase finishes");
    assert_eq!(report.attempts.len(), 2);
    assert_eq!(report.discarded_attempts.len(), 1);
    let discard = &report.discarded_attempts[0];
    assert_eq!(discard.reason, DiscardReason::TransientError);
    assert!(
        discard.detail.contains("physics loop busy"),
        "{}",
        discard.detail
    );
    assert_eq!(state.lock().expect("state").resets, 3);
}

#[test]
fn a_reply_without_link_fails_the_phase_before_any_score() {
    let (port, state) = serve(Scenario {
        outage_at_measure: None,
        outage: Duration::ZERO,
        busy_at_measure: None,
        omit_link_on_reset: true,
    });
    let recovery = RecoveryConfig::new(5.0, 3, 0.02, 0.02, 1.0).expect("recovery");
    let error = run_phase(port, &[1, 2], &recovery).unwrap_err();
    let message = error.to_string();
    assert!(
        matches!(error, TrainerError::Runtime(_))
            && message.contains("reset_episode")
            && message.contains("`link`"),
        "{message}"
    );
    // The very first reset failed: no measure was served, so nothing could have been scored.
    let state = state.lock().expect("state");
    assert_eq!((state.resets, state.measures), (1, 0));
}
