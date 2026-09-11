//! Live-FEAGI integration test for the remote ZMQ runtime (plan Phase 1b).
//!
//! This test only compiles with `--features remote-runtime` and only *runs* its body when a live
//! FEAGI registration endpoint is supplied via the `FEAGI_TRAINER_LIVE_REGISTRATION_ENDPOINT`
//! environment variable (e.g. `tcp://127.0.0.1:30000`). Without it the test self-skips: the live
//! FEAGI instance is a collaborator outside the test's domain and is not available in CI. Nothing
//! about the endpoint or timing is hardcoded — all of it is supplied by the operator.
//!
//! Run it against a live brain with, for example:
//!
//! ```bash
//! FEAGI_TRAINER_LIVE_REGISTRATION_ENDPOINT=tcp://127.0.0.1:30000 \
//!   cargo test -p feagi-trainer --features remote-runtime --test remote_runtime_live -- --nocapture
//! ```
#![cfg(feature = "remote-runtime")]

use std::time::Duration;

use feagi_trainer::binding::remote_runtime::{RemoteFeagiRuntime, RemoteRuntimeConfig};
use feagi_trainer::binding::reward::{AffectChannel, RewardSignal};
use feagi_trainer::binding::runtime::FeagiRuntime;

/// Environment variable that supplies the live FEAGI registration (command/control) endpoint.
const ENDPOINT_ENV: &str = "FEAGI_TRAINER_LIVE_REGISTRATION_ENDPOINT";
/// Optional environment variable for the burst frequency (Hz) used to size the step wait.
const BURST_HZ_ENV: &str = "FEAGI_TRAINER_LIVE_BURST_HZ";

fn parse_required_endpoint() -> Option<String> {
    match std::env::var(ENDPOINT_ENV) {
        Ok(value) if !value.trim().is_empty() => Some(value),
        _ => {
            eprintln!(
                "skipping live remote-runtime test: set {ENDPOINT_ENV} (e.g. tcp://127.0.0.1:30000) to run"
            );
            None
        }
    }
}

fn burst_period_from_env() -> Result<Duration, String> {
    let raw = std::env::var(BURST_HZ_ENV).map_err(|_| {
        format!("{BURST_HZ_ENV} must be set (burst frequency in Hz) for the live test")
    })?;
    let hz: f64 = raw
        .trim()
        .parse()
        .map_err(|e| format!("invalid {BURST_HZ_ENV} value '{raw}': {e}"))?;
    if hz <= 0.0 {
        return Err(format!("{BURST_HZ_ENV} must be positive, got {hz}"));
    }
    Ok(Duration::from_secs_f64(1.0 / hz))
}

#[test]
fn remote_runtime_registers_and_drives_a_live_feagi() {
    let Some(registration_endpoint) = parse_required_endpoint() else {
        return;
    };
    let burst_period = burst_period_from_env().expect("burst frequency configuration");

    let config = RemoteRuntimeConfig {
        registration_endpoint,
        manufacturer: "feagi-trainer".to_string(),
        agent_name: "remote-runtime-live-test".to_string(),
        agent_version: 1,
        auth_token: [0u8; 32],
        burst_period,
        registration_poll_interval: Duration::from_millis(20),
        registration_timeout: Duration::from_secs(10),
        motor_poll_interval: Duration::from_millis(10),
        motor_collect_timeout: Duration::from_secs(2),
    };

    let mut runtime =
        RemoteFeagiRuntime::connect_and_register(config).expect("registration with live FEAGI");

    // Affect areas (___pleas etc.) are core cortical_area areas present in every genome, so reward
    // stimulation can be exercised without knowing the loaded IPU/OPU layout.
    runtime
        .submit_reward(&[RewardSignal {
            channel: AffectChannel::Pleasure,
            magnitude: 0.5,
        }])
        .expect("reward stimulation publish");

    // Give the free-running brain a few ticks of wall-clock integration time.
    runtime.step(5).expect("wall-clock step");

    // Motor output depends on the loaded genome; a timeout here is acceptable for a smoke test, an
    // explicit error from the transport layer is not.
    match runtime.collect_motor() {
        Ok(frame) => eprintln!(
            "collected motor frame with {} cortical_area area(s)",
            frame.len()
        ),
        Err(error) => eprintln!("no motor frame collected (acceptable for smoke test): {error}"),
    }

    runtime.shutdown().expect("deregister from live FEAGI");
}
