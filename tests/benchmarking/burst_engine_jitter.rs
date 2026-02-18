// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Burst engine jitter test under stress.
//!
//! Measures inter-burst interval stability (jitter) of the real burst loop while
//! optionally contending for the NPU lock to simulate API stress.
//!
//! Environment (optional):
//! - FEAGI_BENCH_JITTER_SECS: run duration in seconds (default: 5)
//! - FEAGI_BENCH_JITTER_HZ: target burst frequency (default: 30.0)
//! - FEAGI_BENCH_JITTER_STRESS: "1" to run a contention thread (default: "1")
//!
//! Run: cargo test --release --test benchmarking burst_engine_jitter -- --nocapture
//!
//! Second test: burst_engine_jitter_with_injection runs jitter at 100, 1k, 10k, 100k, 1M
//! sensory injections per burst (staged each burst, drained in Phase 1).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use feagi_npu_burst_engine::backend::CPUBackend;
use feagi_npu_burst_engine::{
    BurstLoopRunner, DynamicNPU, MotorPublisher, RawFireQueueSnapshot, RustNPU, TracingMutex,
    VisualizationPublisher,
};
use feagi_npu_neural::types::NeuronId;
use feagi_npu_runtime::StdRuntime;
use feagi_structures::genomic::cortical_area::CoreCorticalType;

fn read_env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

fn read_env_f64(name: &str, default: f64) -> f64 {
    std::env::var(name)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

fn read_env_stress(name: &str, default: bool) -> bool {
    std::env::var(name)
        .ok()
        .map(|s| s == "1" || s.eq_ignore_ascii_case("true"))
        .unwrap_or(default)
}

struct NoViz;
impl VisualizationPublisher for NoViz {
    fn publish_raw_fire_queue_for_agent(
        &self,
        _agent_id: &str,
        _fire_data: RawFireQueueSnapshot,
    ) -> Result<(), String> {
        Ok(())
    }
}

struct NoMotor;
impl MotorPublisher for NoMotor {
    fn publish_motor(&self, _agent_id: &str, _data: &[u8]) -> Result<(), String> {
        Ok(())
    }
}

#[test]
fn burst_engine_jitter_under_stress() {
    let duration_secs = read_env_u64("FEAGI_BENCH_JITTER_SECS", 5);
    let frequency_hz = read_env_f64("FEAGI_BENCH_JITTER_HZ", 30.0);
    let stress_enabled = read_env_stress("FEAGI_BENCH_JITTER_STRESS", true);

    let target_interval_ms = 1000.0 / frequency_hz;
    let min_intervals = 100;

    let runtime = StdRuntime;
    let backend = CPUBackend::new();
    let rust_npu = RustNPU::new(runtime, backend, 2000, 20_000, 20).expect("create NPU");
    let npu = Arc::new(TracingMutex::new(
        DynamicNPU::F32(rust_npu),
        "JitterTestNPU",
    ));
    let runner = BurstLoopRunner::new::<NoViz, NoMotor>(npu.clone(), None, None, frequency_hz);
    let runner_shared = Arc::new(std::sync::RwLock::new(runner));

    runner_shared
        .write()
        .unwrap()
        .start()
        .expect("burst loop start");

    let timestamps: Arc<Mutex<Vec<Instant>>> = Arc::new(Mutex::new(Vec::new()));
    let timestamps_obs = timestamps.clone();
    let runner_obs = runner_shared.clone();
    let stop = Arc::new(AtomicBool::new(false));
    let stop_obs = stop.clone();

    let observer_handle = thread::spawn(move || {
        let mut last_count: u64 = 0;
        while !stop_obs.load(Ordering::Relaxed) {
            let c = runner_obs.read().unwrap().get_burst_count();
            if c != last_count {
                timestamps_obs.lock().unwrap().push(Instant::now());
                last_count = c;
            }
            thread::sleep(Duration::from_millis(1));
        }
    });

    let stress_handle = if stress_enabled {
        let runner_stress = runner_shared.clone();
        let stop_stress = stop.clone();
        Some(thread::spawn(move || {
            while !stop_stress.load(Ordering::Relaxed) {
                let _ = runner_stress.read().unwrap().get_fcl_snapshot();
                thread::sleep(Duration::from_millis(8));
            }
        }))
    } else {
        None
    };

    let deadline = Instant::now() + Duration::from_secs(duration_secs);
    while deadline > Instant::now() {
        let n = timestamps.lock().unwrap().len();
        if n >= min_intervals + 1 {
            break;
        }
        thread::sleep(Duration::from_millis(50));
    }

    stop.store(true, Ordering::Relaxed);
    observer_handle.join().expect("observer join");
    if let Some(h) = stress_handle {
        h.join().expect("stress join");
    }

    runner_shared.write().unwrap().stop();

    let ts = timestamps.lock().unwrap().clone();
    drop(timestamps);

    assert!(
        ts.len() >= 2,
        "need at least 2 timestamps for intervals, got {}",
        ts.len()
    );

    let intervals_ms: Vec<f64> = ts
        .windows(2)
        .map(|w| w[1].duration_since(w[0]).as_secs_f64() * 1000.0)
        .collect();

    let mean_ms: f64 = intervals_ms.iter().sum::<f64>() / intervals_ms.len() as f64;
    let variance: f64 = intervals_ms
        .iter()
        .map(|x| (x - mean_ms).powi(2))
        .sum::<f64>()
        / intervals_ms.len() as f64;
    let std_ms = variance.sqrt();

    let mut sorted = intervals_ms.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p50_ms = sorted[sorted.len() / 2];
    let p95_idx = (sorted.len() as f64 * 0.95) as usize;
    let p95_idx = p95_idx.min(sorted.len().saturating_sub(1));
    let p95_ms = sorted[p95_idx];
    let p99_idx = (sorted.len() as f64 * 0.99) as usize;
    let p99_idx = p99_idx.min(sorted.len().saturating_sub(1));
    let p99_ms = sorted[p99_idx];

    let cv_percent = if mean_ms > 0.0 {
        (std_ms / mean_ms) * 100.0
    } else {
        0.0
    };

    println!("Burst engine jitter (stress={}):", stress_enabled);
    println!(
        "  Target interval: {:.2} ms ({} Hz)",
        target_interval_ms, frequency_hz
    );
    println!("  Samples: {} intervals", intervals_ms.len());
    println!("  Mean interval:   {:.2} ms", mean_ms);
    println!("  Std dev:         {:.2} ms", std_ms);
    println!("  CV:              {:.1}%", cv_percent);
    println!("  P50:             {:.2} ms", p50_ms);
    println!("  P95:             {:.2} ms", p95_ms);
    println!("  P99:             {:.2} ms", p99_ms);

    let max_cv_percent = 35.0;
    assert!(
        cv_percent <= max_cv_percent,
        "jitter too high: CV {:.1}% > {}%",
        cv_percent,
        max_cv_percent
    );

    let max_p99_ratio = 2.0;
    let p99_ratio = p99_ms / target_interval_ms;
    assert!(
        p99_ratio <= max_p99_ratio,
        "P99 interval {:.2} ms > {}x target ({:.2} ms)",
        p99_ms,
        max_p99_ratio,
        target_interval_ms
    );
}

/// Jitter thresholds per injection load and frequency (CV % and P99 ratio).
/// Higher load and higher frequency may allow more jitter. At 100+ Hz with 1M
/// injections the burst cannot meet target interval (overload), so P99 ratio is very high.
fn max_cv_and_p99_ratio_for_injection(injection_count: usize, frequency_hz: f64) -> (f64, f64) {
    let (cv, ratio) = match injection_count {
        n if n <= 100 => (35.0, 2.0),
        n if n <= 1_000 => (40.0, 2.2),
        n if n <= 10_000 => (50.0, 2.5),
        n if n <= 100_000 => (60.0, 3.0),
        _ => (95.0, 25.0), // 1M: overload at high Hz, allow very high jitter
    };
    if frequency_hz >= 100.0 {
        let ratio_bonus = match injection_count {
            n if n >= 1_000_000 => 150.0,
            n if n >= 100_000 => 15.0,
            n if n >= 10_000 => 2.0,
            _ => 1.0,
        };
        (cv + 25.0, ratio + ratio_bonus)
    } else if frequency_hz >= 30.0 {
        (cv + 5.0, ratio + 0.3)
    } else {
        (cv, ratio)
    }
}

/// Duration in seconds to run: enough to get min_intervals at this frequency.
/// Longer at high injection counts (bursts take longer).
fn duration_secs_for_run(frequency_hz: f64, injection_count: usize) -> u64 {
    let min_bursts = 80u64;
    let secs_for_bursts = (min_bursts as f64 / frequency_hz).ceil().max(1.0) as u64;
    let extra_for_heavy = match injection_count {
        n if n <= 10_000 => 0,
        n if n <= 100_000 => 8,
        _ => 20,
    };
    (secs_for_bursts + extra_for_heavy).max(2)
}

#[test]
fn burst_engine_jitter_with_injection() {
    let frequency_levels: [f64; 4] = [15.0, 30.0, 100.0, 1000.0];
    let injection_levels: [usize; 5] = [100, 1000, 10_000, 100_000, 1_000_000];

    println!("\nHz\tInj\tSamples\tMean_ms\tStd_ms\tCV%\tP99_ms\tP99x");

    for &frequency_hz in &frequency_levels {
        let target_interval_ms = 1000.0 / frequency_hz;

        for &injection_count in &injection_levels {
            let neuron_capacity = injection_count + 32;
            let synapse_capacity = (injection_count * 2).max(10_000);
            let runtime = StdRuntime;
            let backend = CPUBackend::new();
            let mut rust_npu =
                RustNPU::new(runtime, backend, neuron_capacity, synapse_capacity, 20)
                    .expect("create NPU");

            rust_npu
                .register_cortical_area(0, CoreCorticalType::Death.to_cortical_id().as_base_64());
            rust_npu
                .register_cortical_area(1, CoreCorticalType::Power.to_cortical_id().as_base_64());
            rust_npu
                .register_cortical_area(2, CoreCorticalType::Death.to_cortical_id().as_base_64());
            rust_npu
                .register_cortical_area(10, CoreCorticalType::Death.to_cortical_id().as_base_64());

            if injection_count <= 1000 {
                for i in 0..injection_count {
                    let _ = rust_npu.add_neuron(
                        1.0,
                        f32::MAX,
                        0.1,
                        0.0,
                        0,
                        5,
                        1.0,
                        0,
                        0,
                        true,
                        10,
                        i as u32,
                        0,
                        0,
                    );
                }
            } else {
                let n = injection_count;
                let (added, _failed) = rust_npu.add_neurons_batch(
                    vec![1.0f32; n],
                    vec![f32::MAX; n],
                    vec![0.1f32; n],
                    vec![0.0f32; n],
                    vec![0i32; n],
                    vec![5u16; n],
                    vec![1.0f32; n],
                    vec![0u16; n],
                    vec![0u16; n],
                    vec![true; n],
                    vec![10u32; n],
                    (0..n).map(|i| i as u32).collect(),
                    vec![0u32; n],
                    vec![0u32; n],
                );
                assert!(
                    added as usize == n,
                    "add_neurons_batch: added {} expected {}",
                    added,
                    n
                );
            }

            let npu = Arc::new(TracingMutex::new(
                DynamicNPU::F32(rust_npu),
                "JitterInjectionNPU",
            ));
            let sensory_list: Vec<(NeuronId, f32)> = (3..3 + injection_count)
                .map(|i| (NeuronId(i as u32), 100.0))
                .collect();

            let runner =
                BurstLoopRunner::new::<NoViz, NoMotor>(npu.clone(), None, None, frequency_hz);
            let runner_shared = Arc::new(std::sync::RwLock::new(runner));

            runner_shared
                .write()
                .unwrap()
                .start()
                .expect("burst loop start");

            let timestamps: Arc<Mutex<Vec<Instant>>> = Arc::new(Mutex::new(Vec::new()));
            let timestamps_obs = timestamps.clone();
            let runner_obs = runner_shared.clone();
            let npu_obs = npu.clone();
            let sensory_obs = sensory_list.clone();
            let stop = Arc::new(AtomicBool::new(false));
            let stop_obs = stop.clone();

            let duration_secs = duration_secs_for_run(frequency_hz, injection_count);
            let min_intervals = 80;

            let observer_handle = thread::spawn(move || {
                let mut last_count: u64 = 0;
                while !stop_obs.load(Ordering::Relaxed) {
                    let c = runner_obs.read().unwrap().get_burst_count();
                    if c != last_count {
                        timestamps_obs.lock().unwrap().push(Instant::now());
                        if last_count > 0 {
                            if let Ok(mut guard) = npu_obs.lock() {
                                guard.inject_sensory_with_potentials(&sensory_obs);
                            }
                        }
                        last_count = c;
                    }
                    thread::sleep(Duration::from_millis(1));
                }
            });

            let deadline = Instant::now() + Duration::from_secs(duration_secs);
            while deadline > Instant::now() {
                let n = timestamps.lock().unwrap().len();
                if n >= min_intervals + 1 {
                    break;
                }
                thread::sleep(Duration::from_millis(100));
            }

            stop.store(true, Ordering::Relaxed);
            observer_handle.join().expect("observer join");
            runner_shared.write().unwrap().stop();

            let ts = timestamps.lock().unwrap().clone();
            assert!(
                ts.len() >= 2,
                "{} Hz injection {}: need at least 2 timestamps, got {}",
                frequency_hz,
                injection_count,
                ts.len()
            );

            let intervals_ms: Vec<f64> = ts
                .windows(2)
                .map(|w| w[1].duration_since(w[0]).as_secs_f64() * 1000.0)
                .collect();

            let mean_ms: f64 = intervals_ms.iter().sum::<f64>() / intervals_ms.len() as f64;
            let variance: f64 = intervals_ms
                .iter()
                .map(|x| (x - mean_ms).powi(2))
                .sum::<f64>()
                / intervals_ms.len() as f64;
            let std_ms = variance.sqrt();

            let mut sorted = intervals_ms.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let p99_idx = (sorted.len() as f64 * 0.99) as usize;
            let p99_idx = p99_idx.min(sorted.len().saturating_sub(1));
            let p99_ms = sorted[p99_idx];

            let cv_percent = if mean_ms > 0.0 {
                (std_ms / mean_ms) * 100.0
            } else {
                0.0
            };

            let (max_cv, max_p99_ratio) =
                max_cv_and_p99_ratio_for_injection(injection_count, frequency_hz);
            let p99_ratio = p99_ms / target_interval_ms;

            println!(
                "{}\t{}\t{}\t{:.2}\t{:.2}\t{:.1}\t{:.2}\t{:.2}",
                frequency_hz,
                injection_count,
                intervals_ms.len(),
                mean_ms,
                std_ms,
                cv_percent,
                p99_ms,
                p99_ratio
            );

            assert!(
                cv_percent <= max_cv,
                "{} Hz injection {}: CV {:.1}% > {}%",
                frequency_hz,
                injection_count,
                cv_percent,
                max_cv
            );
            assert!(
                p99_ratio <= max_p99_ratio,
                "{} Hz injection {}: P99 ratio {:.2} > {}",
                frequency_hz,
                injection_count,
                p99_ratio,
                max_p99_ratio
            );
        }
    }

    println!("\nAll frequency and injection levels passed.");
}
