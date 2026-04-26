// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Runtime taps - lightweight introspection points for the burst loop.
//!
//! These taps record the most recent motor output and sensory input data observed
//! by the burst loop. They exist to support debugging, diagnostics, and external
//! tooling (FEAGI MCP server, Brain Visualizer, third-party debuggers) that needs
//! to inspect what FEAGI is actually exchanging with connected embodiments.
//!
//! Design notes:
//! - A global [`BurstTaps`] singleton is reachable through [`BurstTaps::instance`]
//!   so the burst loop can write without threading additional `Arc` references
//!   through the runner constructor.
//! - Writes happen at most once per burst from the burst loop hot path; data is
//!   accumulated in local buffers and committed under a single `parking_lot::RwLock`
//!   write so the lock window is sub-microsecond.
//! - Reads happen from REST handlers at far lower frequency.
//! - Only the most recent burst is retained, which keeps memory cost bounded
//!   regardless of burst frequency.

use ahash::AHashMap;
use parking_lot::RwLock;
use std::sync::OnceLock;

/// One sample of activity within a tap-captured cortical area:
/// voxel coordinate plus normalised potential (0.0..=1.0).
#[derive(Debug, Clone, Default)]
pub struct TapSample {
    pub x: u32,
    pub y: u32,
    pub z: u32,
    pub potential: f32,
}

/// Activity captured for a single cortical area within one burst.
#[derive(Debug, Clone, Default)]
pub struct AreaActivity {
    /// Base64 cortical ID (matches the public REST representation).
    pub cortical_id: String,
    /// Internal NPU cortical index (useful for cross-referencing low-level traces).
    pub cortical_idx: u32,
    /// Number of neurons that fired in this area for this burst.
    pub neuron_count: usize,
    /// Per-neuron firing samples.
    pub samples: Vec<TapSample>,
}

/// Most recent motor publish event for a specific agent.
#[derive(Debug, Clone, Default)]
pub struct AgentPublishStats {
    /// Burst number when this publish occurred.
    pub burst_num: u64,
    /// Wall-clock millisecond timestamp when this publish occurred.
    pub timestamp_ms: i64,
    /// Number of bytes published (after motor encoding).
    pub byte_count: usize,
    /// True if the publish to the configured transport succeeded.
    pub published: bool,
    /// Last error message (empty when publish succeeded).
    pub last_error: String,
    /// Cortical IDs the agent was subscribed to at publish time.
    pub subscribed_cortical_ids: Vec<String>,
}

/// Most recent motor output produced by the burst loop.
#[derive(Debug, Clone, Default)]
pub struct MotorOutputTap {
    /// Burst counter at the moment the area snapshot was captured.
    /// Defaults to 0 when the tap has never been updated.
    pub burst_num: u64,
    /// Wall-clock millisecond timestamp for the area snapshot.
    pub timestamp_ms: i64,
    /// Per-cortical-area activity in the motor pipeline. This is the unfiltered
    /// view (what the motor stage sees before per-agent subscription filtering).
    pub areas: Vec<AreaActivity>,
    /// Per-agent publish summaries keyed by agent_id. Entries are updated only
    /// for agents that actually published in a given burst, so older agents
    /// retain their last-known stats until they publish again.
    pub per_agent: AHashMap<String, AgentPublishStats>,
}

/// Most recent sensory input observed by the burst loop.
#[derive(Debug, Clone, Default)]
pub struct SensorInputTap {
    /// Burst counter at the moment the snapshot was captured.
    pub burst_num: u64,
    /// Wall-clock millisecond timestamp for the snapshot.
    pub timestamp_ms: i64,
    /// Per-cortical-area decoded sensory input for the burst.
    pub areas: Vec<AreaActivity>,
}

/// Combined runtime taps available globally.
pub struct BurstTaps {
    pub motor: RwLock<MotorOutputTap>,
    pub sensor: RwLock<SensorInputTap>,
}

static BURST_TAPS: OnceLock<BurstTaps> = OnceLock::new();

impl BurstTaps {
    /// Lazily initialise and return a reference to the global tap instance.
    pub fn instance() -> &'static BurstTaps {
        BURST_TAPS.get_or_init(|| BurstTaps {
            motor: RwLock::new(MotorOutputTap::default()),
            sensor: RwLock::new(SensorInputTap::default()),
        })
    }

    /// Read the latest motor snapshot (clones into the caller).
    pub fn motor_snapshot(&self) -> MotorOutputTap {
        self.motor.read().clone()
    }

    /// Read the latest sensor snapshot (clones into the caller).
    pub fn sensor_snapshot(&self) -> SensorInputTap {
        self.sensor.read().clone()
    }
}

/// Convenience helper - returns the current Unix time in milliseconds.
/// Returns 0 when the system clock is misconfigured rather than panicking.
pub fn now_unix_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instance_is_idempotent() {
        let a = BurstTaps::instance() as *const BurstTaps;
        let b = BurstTaps::instance() as *const BurstTaps;
        assert_eq!(a, b, "BurstTaps::instance must return the same singleton");
    }

    #[test]
    fn motor_tap_round_trip() {
        let taps = BurstTaps::instance();
        {
            let mut motor = taps.motor.write();
            motor.burst_num = 42;
            motor.timestamp_ms = 1_700_000_000_000;
            motor.areas.clear();
            motor.areas.push(AreaActivity {
                cortical_id: "test01==".to_string(),
                cortical_idx: 7,
                neuron_count: 3,
                samples: vec![TapSample {
                    x: 1,
                    y: 2,
                    z: 3,
                    potential: 0.5,
                }],
            });
            motor.per_agent.insert(
                "agent-1".to_string(),
                AgentPublishStats {
                    burst_num: 42,
                    timestamp_ms: 1_700_000_000_000,
                    byte_count: 16,
                    published: true,
                    last_error: String::new(),
                    subscribed_cortical_ids: vec!["test01==".to_string()],
                },
            );
        }

        let snap = taps.motor_snapshot();
        assert_eq!(snap.burst_num, 42);
        assert_eq!(snap.areas.len(), 1);
        assert_eq!(snap.areas[0].cortical_id, "test01==");
        let agent = snap.per_agent.get("agent-1").expect("agent stats present");
        assert_eq!(agent.byte_count, 16);
        assert!(agent.published);
    }

    #[test]
    fn sensor_tap_round_trip() {
        let taps = BurstTaps::instance();
        {
            let mut sensor = taps.sensor.write();
            sensor.burst_num = 17;
            sensor.timestamp_ms = 1_700_000_000_000;
            sensor.areas.clear();
            sensor.areas.push(AreaActivity {
                cortical_id: "imu1AAA=".to_string(),
                cortical_idx: 3,
                neuron_count: 2,
                samples: vec![
                    TapSample {
                        x: 0,
                        y: 0,
                        z: 0,
                        potential: 0.9,
                    },
                    TapSample {
                        x: 0,
                        y: 0,
                        z: 1,
                        potential: 0.7,
                    },
                ],
            });
        }
        let snap = taps.sensor_snapshot();
        assert_eq!(snap.burst_num, 17);
        assert_eq!(snap.areas.len(), 1);
        assert_eq!(snap.areas[0].samples.len(), 2);
    }
}
