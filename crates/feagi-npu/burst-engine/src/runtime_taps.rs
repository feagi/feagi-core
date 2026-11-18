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
    /// Cumulative frames successfully published to this agent (ZMQ or SHM; each frame once).
    pub frames_published: u64,
    /// Cumulative trailing empty frames ("output ended") among `frames_published`.
    pub trailing_empty_frames: u64,
}

impl AgentPublishStats {
    /// Set the cumulative counters: the previous totals plus this attempt, counted only when
    /// `published` (a failed attempt adds nothing). `trailing` marks an "output ended" frame.
    pub fn carry_counters_from(
        &mut self,
        previous: Option<&AgentPublishStats>,
        published: bool,
        trailing: bool,
    ) {
        let (frames, trailing_total) =
            previous.map_or((0, 0), |p| (p.frames_published, p.trailing_empty_frames));
        self.frames_published = frames + u64::from(published);
        self.trailing_empty_frames = trailing_total + u64::from(published && trailing);
    }
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
    /// Per-source frame-merge statistics, cumulative since FEAGI start (or the last
    /// [`BurstTaps::clear`]). Keyed by source id; payloads without one use
    /// [`UNKNOWN_SOURCE`]. Unlike `areas`, this is updated every burst.
    pub frame_stats: AHashMap<String, SourceFrameStats>,
}

/// Bucket name for sensory payloads that carry no source id.
pub const UNKNOWN_SOURCE: &str = "unknown";

/// What one source delivered to one burst (built by the burst loop from borrowed ids).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BurstSourceTally {
    /// Payloads that met the burst deadline and entered the merge path.
    pub fresh: u32,
    /// Payloads dropped because they missed the burst deadline.
    pub stale: u32,
    /// Payloads taken by the ordered/audio sequential path (not in the histogram).
    pub sequential: u32,
}

/// How often FEAGI merged several sensory frames from one source into a single burst.
///
/// The histogram covers the newest-wins/merge path only (`fresh` payloads per burst). Frames
/// taken by the ordered/audio sequential path are counted separately in
/// `sequential_frames_received`; a source that has ever sent such frames is excluded from the
/// histogram's zero bucket, because its frames are not expected on the merge path.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SourceFrameStats {
    /// Bursts where this known source contributed no fresh frame.
    pub bursts_with_0_frames: u64,
    /// Bursts where it contributed exactly one fresh frame.
    pub bursts_with_1_frame: u64,
    /// Bursts where it contributed two or more fresh frames (merged into one).
    pub bursts_with_2plus_frames: u64,
    /// Largest number of fresh frames contributed to a single burst.
    pub max_frames_in_one_burst: u32,
    /// All payloads received from this source on the merge path (fresh + stale).
    pub frames_received: u64,
    /// Payloads dropped as stale.
    pub frames_dropped_stale: u64,
    /// Fresh frames contributed to the current (latest) burst.
    pub current_burst_frames: u32,
    /// Payloads taken by the ordered/audio sequential path (outside the histogram).
    pub sequential_frames_received: u64,
}

impl SensorInputTap {
    /// Fold one burst's per-source tallies into the cumulative statistics. Sources seen
    /// before but absent from `tallies` get a zero-frame burst. Integer adds only.
    pub fn record_burst_frames(&mut self, tallies: &AHashMap<String, BurstSourceTally>) {
        for id in tallies.keys() {
            if !self.frame_stats.contains_key(id.as_str()) {
                self.frame_stats
                    .insert(id.clone(), SourceFrameStats::default());
            }
        }
        for (id, st) in self.frame_stats.iter_mut() {
            let t = tallies.get(id.as_str()).copied().unwrap_or_default();
            st.frames_received += u64::from(t.fresh) + u64::from(t.stale);
            st.frames_dropped_stale += u64::from(t.stale);
            st.sequential_frames_received += u64::from(t.sequential);
            st.current_burst_frames = t.fresh;
            st.max_frames_in_one_burst = st.max_frames_in_one_burst.max(t.fresh);
            match t.fresh {
                0 => {
                    if st.sequential_frames_received == 0 {
                        st.bursts_with_0_frames += 1;
                    }
                }
                1 => st.bursts_with_1_frame += 1,
                _ => st.bursts_with_2plus_frames += 1,
            }
        }
    }
}

/// Read-only aggregate activity statistics derived from a single burst's tap snapshot.
///
/// This is a *pure projection* of data the taps already capture: it is computed on the
/// read side (REST/diagnostics) from an existing [`AreaActivity`] slice and introduces no
/// new work on the burst hot path and no change to firing behaviour. It exists so encoder/
/// substrate benchmarking can read spike-cost and timing summaries without each consumer
/// re-deriving them. Only quantities unambiguously present in the tap data are reported;
/// notions that require area capacity (e.g. fired/total occupancy) are intentionally not
/// invented here.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BurstActivitySummary {
    /// Burst counter of the snapshot these statistics were derived from.
    pub burst_num: u64,
    /// Wall-clock millisecond timestamp of the snapshot.
    pub timestamp_ms: i64,
    /// Number of captured cortical areas that fired at least one neuron this burst.
    pub active_area_count: usize,
    /// Total fired neurons across all captured areas (sum of `AreaActivity::neuron_count`).
    pub total_fired_neurons: usize,
    /// Largest single-area fired-neuron count this burst.
    pub peak_area_fired_neurons: usize,
    /// Mean fired neurons per active area (`0.0` when no area fired).
    pub mean_area_fired_neurons: f64,
    /// Mean potential across all captured firing samples (`0.0` when there are none).
    pub mean_sample_potential: f64,
    /// Peak potential across all captured firing samples (`0.0` when there are none).
    pub peak_sample_potential: f32,
}

impl BurstActivitySummary {
    /// Derives the summary from a burst's captured per-area activity.
    ///
    /// `areas` is the unfiltered per-area activity slice from a [`MotorOutputTap`] or
    /// [`SensorInputTap`] snapshot. An area counts as active when its `neuron_count > 0`.
    pub fn from_areas(burst_num: u64, timestamp_ms: i64, areas: &[AreaActivity]) -> Self {
        let mut active_area_count = 0usize;
        let mut total_fired_neurons = 0usize;
        let mut peak_area_fired_neurons = 0usize;
        let mut potential_sum = 0.0f64;
        let mut sample_count = 0usize;
        let mut peak_sample_potential = 0.0f32;

        for area in areas {
            if area.neuron_count > 0 {
                active_area_count += 1;
            }
            total_fired_neurons += area.neuron_count;
            peak_area_fired_neurons = peak_area_fired_neurons.max(area.neuron_count);
            for sample in &area.samples {
                potential_sum += sample.potential as f64;
                sample_count += 1;
                peak_sample_potential = peak_sample_potential.max(sample.potential);
            }
        }

        let mean_area_fired_neurons = if active_area_count > 0 {
            total_fired_neurons as f64 / active_area_count as f64
        } else {
            0.0
        };
        let mean_sample_potential = if sample_count > 0 {
            potential_sum / sample_count as f64
        } else {
            0.0
        };

        Self {
            burst_num,
            timestamp_ms,
            active_area_count,
            total_fired_neurons,
            peak_area_fired_neurons,
            mean_area_fired_neurons,
            mean_sample_potential,
            peak_sample_potential,
        }
    }
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

    /// Drop the latest motor and sensor snapshots.
    ///
    /// Called when a genome is loaded so diagnostics do not keep the previous
    /// brain's last burst.
    pub fn clear(&self) {
        *self.motor.write() = MotorOutputTap::default();
        *self.sensor.write() = SensorInputTap::default();
    }

    /// Read the latest motor snapshot (clones into the caller).
    pub fn motor_snapshot(&self) -> MotorOutputTap {
        self.motor.read().clone()
    }

    /// Read the latest sensor snapshot (clones into the caller).
    pub fn sensor_snapshot(&self) -> SensorInputTap {
        self.sensor.read().clone()
    }

    /// Derives the read-only activity summary for the latest motor snapshot.
    pub fn motor_activity_summary(&self) -> BurstActivitySummary {
        let motor = self.motor.read();
        BurstActivitySummary::from_areas(motor.burst_num, motor.timestamp_ms, &motor.areas)
    }

    /// Derives the read-only activity summary for the latest sensor snapshot.
    pub fn sensor_activity_summary(&self) -> BurstActivitySummary {
        let sensor = self.sensor.read();
        BurstActivitySummary::from_areas(sensor.burst_num, sensor.timestamp_ms, &sensor.areas)
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
    fn clear_drops_motor_and_sensor_snapshots() {
        let taps = BurstTaps::instance();
        {
            let mut motor = taps.motor.write();
            motor.burst_num = 9;
            motor.areas.push(AreaActivity {
                cortical_id: "old".to_string(),
                cortical_idx: 7,
                neuron_count: 1,
                samples: Vec::new(),
            });
        }
        {
            let mut sensor = taps.sensor.write();
            sensor.burst_num = 9;
            sensor.areas.push(AreaActivity {
                cortical_id: "old".to_string(),
                cortical_idx: 8,
                neuron_count: 1,
                samples: Vec::new(),
            });
        }
        taps.clear();
        let motor = taps.motor_snapshot();
        let sensor = taps.sensor_snapshot();
        assert_eq!(motor.burst_num, 0);
        assert!(motor.areas.is_empty());
        assert_eq!(sensor.burst_num, 0);
        assert!(sensor.areas.is_empty());
    }

    #[test]
    fn frame_stats_count_merge_zero_and_stale() {
        let mut tap = SensorInputTap::default();
        let burst = |fresh: u32, stale: u32| {
            let mut m: AHashMap<String, BurstSourceTally> = AHashMap::new();
            m.insert(
                "src".to_string(),
                BurstSourceTally {
                    fresh,
                    stale,
                    sequential: 0,
                },
            );
            m
        };
        tap.record_burst_frames(&burst(1, 0));
        tap.record_burst_frames(&burst(2, 0));
        tap.record_burst_frames(&AHashMap::new()); // known source, no frame
        tap.record_burst_frames(&burst(0, 1)); // only a stale frame
        let st = &tap.frame_stats["src"];
        assert_eq!(st.bursts_with_1_frame, 1);
        assert_eq!(st.bursts_with_2plus_frames, 1);
        assert_eq!(st.bursts_with_0_frames, 2);
        assert_eq!(st.max_frames_in_one_burst, 2);
        assert_eq!(st.frames_received, 4);
        assert_eq!(st.frames_dropped_stale, 1);
        assert_eq!(st.current_burst_frames, 0);
    }

    #[test]
    fn frame_stats_sequential_source_has_no_zero_bursts() {
        let mut tap = SensorInputTap::default();
        let mut m: AHashMap<String, BurstSourceTally> = AHashMap::new();
        m.insert(
            "aud".to_string(),
            BurstSourceTally {
                fresh: 0,
                stale: 0,
                sequential: 2,
            },
        );
        tap.record_burst_frames(&m);
        tap.record_burst_frames(&AHashMap::new());
        let st = &tap.frame_stats["aud"];
        assert_eq!(st.bursts_with_0_frames, 0);
        assert_eq!(st.sequential_frames_received, 2);
    }

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
                    frames_published: 5,
                    trailing_empty_frames: 2,
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
        assert_eq!(agent.frames_published, 5);
        assert_eq!(agent.trailing_empty_frames, 2);
    }

    #[test]
    fn publish_counters_accumulate_across_commits() {
        let mut previous = AgentPublishStats::default();
        // (published, trailing) per commit: real frame, failed attempt, trailing frame.
        for (published, trailing) in [(true, false), (false, false), (true, true)] {
            let mut next = AgentPublishStats {
                published,
                ..AgentPublishStats::default()
            };
            next.carry_counters_from(Some(&previous), published, trailing);
            previous = next;
        }
        assert_eq!(previous.frames_published, 2);
        assert_eq!(previous.trailing_empty_frames, 1);
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

    fn area(cortical_id: &str, neuron_count: usize, potentials: &[f32]) -> AreaActivity {
        AreaActivity {
            cortical_id: cortical_id.to_string(),
            cortical_idx: 0,
            neuron_count,
            samples: potentials
                .iter()
                .enumerate()
                .map(|(i, &p)| TapSample {
                    x: i as u32,
                    y: 0,
                    z: 0,
                    potential: p,
                })
                .collect(),
        }
    }

    #[test]
    fn activity_summary_empty_is_zeroed() {
        let summary = BurstActivitySummary::from_areas(9, 1_700_000_000_000, &[]);
        assert_eq!(summary.burst_num, 9);
        assert_eq!(summary.active_area_count, 0);
        assert_eq!(summary.total_fired_neurons, 0);
        assert_eq!(summary.peak_area_fired_neurons, 0);
        assert_eq!(summary.mean_area_fired_neurons, 0.0);
        assert_eq!(summary.mean_sample_potential, 0.0);
        assert_eq!(summary.peak_sample_potential, 0.0);
    }

    #[test]
    fn activity_summary_aggregates_counts_and_potentials() {
        let areas = vec![
            area("aaaaAA==", 3, &[0.2, 0.4, 0.6]),
            area("bbbbAA==", 1, &[1.0]),
        ];
        let summary = BurstActivitySummary::from_areas(5, 42, &areas);

        assert_eq!(summary.active_area_count, 2);
        assert_eq!(summary.total_fired_neurons, 4);
        assert_eq!(summary.peak_area_fired_neurons, 3);
        assert_eq!(summary.mean_area_fired_neurons, 2.0);
        // (0.2 + 0.4 + 0.6 + 1.0) / 4 = 0.55 (f32 inputs -> f32-grade tolerance)
        assert!((summary.mean_sample_potential - 0.55).abs() < 1e-6);
        assert_eq!(summary.peak_sample_potential, 1.0);
    }

    #[test]
    fn activity_summary_ignores_silent_areas_in_active_count() {
        let areas = vec![area("aaaaAA==", 0, &[]), area("bbbbAA==", 2, &[0.5, 0.5])];
        let summary = BurstActivitySummary::from_areas(1, 1, &areas);

        assert_eq!(summary.active_area_count, 1);
        assert_eq!(summary.total_fired_neurons, 2);
        assert_eq!(summary.mean_area_fired_neurons, 2.0);
    }
}
