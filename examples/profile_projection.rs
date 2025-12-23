// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/// Standalone profiling tool for projection performance analysis
///
/// This creates a minimal genome with power00 area (128x128) and optionally
/// iic400 area with projection, then measures burst performance.
///
/// Run with:
/// ```bash
/// cd feagi-core
/// cargo run --release --example profile_projection
/// ```
///
/// Note: Edit the constants below to change test parameters
// Note: parking_lot removed, using std::sync::Mutex
use std::sync::Arc;
use std::time::{Duration, Instant};

use feagi_bdu::Neuroembryogenesis;
use feagi_evolutionary::{templates, RuntimeGenome};
use feagi_npu_burst_engine::DynamicNPU;
use feagi_structures::genomic::cortical_area::{
    AreaType, CorticalArea, CorticalAreaDimensions as Dimensions, CorticalID,
};

// Configuration - edit these to change test parameters
const WITH_PROJECTION: bool = true; // Set to true to test with projection
const NUM_BURSTS: usize = 300;
const TARGET_FREQUENCY_HZ: f32 = 15.0;
const STIMULUS_NEURONS: usize = 500;

fn main() {
    println!("\n{}", "=".repeat(80));
    println!("FEAGI PROJECTION PERFORMANCE PROFILING");
    println!("{}", "=".repeat(80));
    println!();
    println!("Configuration:");
    println!("  Projection enabled: {}", WITH_PROJECTION);
    println!("  Bursts to run:      {}", NUM_BURSTS);
    println!(
        "  Target frequency:   {} Hz ({:.1} ms/burst)",
        TARGET_FREQUENCY_HZ,
        1000.0 / TARGET_FREQUENCY_HZ
    );
    println!("  Stimulus neurons:   {}", STIMULUS_NEURONS);
    println!();

    // Run profiling
    let metrics = run_profiling(
        WITH_PROJECTION,
        NUM_BURSTS,
        TARGET_FREQUENCY_HZ,
        STIMULUS_NEURONS,
    );

    // Print results
    println!("\n{}", "=".repeat(80));
    println!("RESULTS");
    println!("{}", "=".repeat(80));
    println!();

    print_metrics(&metrics, TARGET_FREQUENCY_HZ);

    // Bottleneck analysis
    println!("\n{}", "=".repeat(80));
    println!("BOTTLENECK ANALYSIS");
    println!("{}", "=".repeat(80));
    println!();

    analyze_bottlenecks(&metrics, TARGET_FREQUENCY_HZ);
}

fn run_profiling(
    with_projection: bool,
    num_bursts: usize,
    target_hz: f32,
    stimulus_count: usize,
) -> PerformanceMetrics {
    const AREA_SIZE: (u32, u32, u32) = (128, 128, 1);

    println!("Creating genome...");
    let genome = create_genome(with_projection, AREA_SIZE);

    println!("Initializing BDU and loading genome...");
    let bdu = Arc::new(Mutex::new(BrainDevelopmentUnit::new()));

    {
        let mut bdu_lock = bdu.lock();
        bdu_lock
            .load_from_genome(genome)
            .expect("Failed to load genome");
    }

    let npu = {
        let bdu_lock = bdu.lock();
        bdu_lock.get_npu().expect("Failed to get NPU")
    };

    println!("✓ Genome loaded");
    println!();

    // Warmup
    println!("Warming up (50 bursts)...");
    for _ in 0..50 {
        inject_stimulus(&npu, stimulus_count, AREA_SIZE);
        let mut npu_lock = npu.lock();
        npu_lock.process_burst().expect("Burst failed");
    }
    println!("✓ Warmup complete");
    println!();

    // Profiling run
    println!("Running profiling ({} bursts)...", num_bursts);
    let mut metrics = PerformanceMetrics::new();
    let test_start = Instant::now();

    for burst_num in 0..num_bursts {
        let burst_start = Instant::now();

        // Inject
        let inject_start = Instant::now();
        inject_stimulus(&npu, stimulus_count, AREA_SIZE);
        let inject_duration = inject_start.elapsed();

        // Process burst
        let exec_start = Instant::now();
        {
            let mut npu_lock = npu.lock();
            npu_lock.process_burst().expect("Burst failed");
        }
        let exec_duration = exec_start.elapsed();

        let burst_duration = burst_start.elapsed();
        metrics.record_burst(burst_duration, inject_duration, exec_duration);

        // Progress
        if (burst_num + 1) % 50 == 0 {
            let progress = ((burst_num + 1) as f32 / num_bursts as f32) * 100.0;
            println!(
                "  Progress: {:.1}% ({}/{})",
                progress,
                burst_num + 1,
                num_bursts
            );
        }
    }

    metrics.total_duration = test_start.elapsed();
    println!("✓ Profiling complete");

    metrics
}

fn create_genome(with_projection: bool, area_size: (u32, u32, u32)) -> RuntimeGenome {
    let mut genome = templates::create_genome_with_core_areas(
        "profiling-test".to_string(),
        "Projection Performance Test".to_string(),
    );

    templates::add_core_morphologies(&mut genome.morphologies);

    // Create power00
    let power_area = CorticalArea::new(
        "power00".to_string(),
        2,
        "Power".to_string(),
        Dimensions::new(
            area_size.0 as usize,
            area_size.1 as usize,
            area_size.2 as usize,
        ),
        (0, 0, 0),
        AreaType::Memory,
    )
    .expect("Failed to create power area");

    genome
        .cortical_areas
        .insert("power00".to_string(), power_area);

    if with_projection {
        let iic400 = CorticalArea::new(
            "iic400".to_string(),
            3,
            "Vision Input".to_string(),
            Dimensions::new(
                area_size.0 as usize,
                area_size.1 as usize,
                area_size.2 as usize,
            ),
            (10, 0, 0),
            AreaType::Sensory, // Use Sensory for visual input areas
        )
        .expect("Failed to create iic400");

        genome.cortical_areas.insert("iic400".to_string(), iic400);
    }

    genome
}

fn inject_stimulus(npu: &Arc<Mutex<DynamicNPU>>, neuron_count: usize, area_size: (u32, u32, u32)) {
    // Create a simple fixed pattern for stimulus (avoid rand dependency)
    // Just iterate through neurons with a stride to get good distribution
    let max_id = (area_size.0 * area_size.1) as usize;
    let stride = max_id / neuron_count.max(1);

    let mut xyzp_data = Vec::with_capacity(neuron_count);
    for i in 0..neuron_count {
        let flat_id = ((i * stride) % max_id) as u32;
        let x = flat_id % area_size.0;
        let y = flat_id / area_size.0;
        let z = 0;
        let potential = 75.0; // Fixed potential for simplicity
        xyzp_data.push((x, y, z, potential));
    }

    let mut npu_lock = npu.lock();
    npu_lock.inject_sensory_xyzp("power00", &xyzp_data);
}

#[derive(Debug)]
struct PerformanceMetrics {
    burst_times: Vec<Duration>,
    inject_times: Vec<Duration>,
    exec_times: Vec<Duration>,
    total_duration: Duration,
}

impl PerformanceMetrics {
    fn new() -> Self {
        Self {
            burst_times: Vec::new(),
            inject_times: Vec::new(),
            exec_times: Vec::new(),
            total_duration: Duration::ZERO,
        }
    }

    fn record_burst(&mut self, total: Duration, inject: Duration, exec: Duration) {
        self.burst_times.push(total);
        self.inject_times.push(inject);
        self.exec_times.push(exec);
    }

    fn avg(&self, times: &[Duration]) -> Duration {
        let sum: Duration = times.iter().sum();
        sum / times.len() as u32
    }

    fn percentile(&self, times: &[Duration], p: f32) -> Duration {
        let mut sorted = times.to_vec();
        sorted.sort();
        let idx = (sorted.len() as f32 * p) as usize;
        sorted[idx.min(sorted.len() - 1)]
    }
}

fn print_metrics(metrics: &PerformanceMetrics, target_hz: f32) {
    let avg_burst = metrics.avg(&metrics.burst_times).as_secs_f32() * 1000.0;
    let avg_inject = metrics.avg(&metrics.inject_times).as_secs_f32() * 1000.0;
    let avg_exec = metrics.avg(&metrics.exec_times).as_secs_f32() * 1000.0;

    let min_burst = metrics.burst_times.iter().min().unwrap().as_secs_f32() * 1000.0;
    let max_burst = metrics.burst_times.iter().max().unwrap().as_secs_f32() * 1000.0;
    let p95_burst = metrics.percentile(&metrics.burst_times, 0.95).as_secs_f32() * 1000.0;
    let p99_burst = metrics.percentile(&metrics.burst_times, 0.99).as_secs_f32() * 1000.0;

    let achieved_hz = metrics.burst_times.len() as f32 / metrics.total_duration.as_secs_f32();

    println!(
        "Total Duration:        {:.2} s",
        metrics.total_duration.as_secs_f32()
    );
    println!("Total Bursts:          {}", metrics.burst_times.len());
    println!(
        "Achieved Frequency:    {:.2} Hz (target: {:.1} Hz)",
        achieved_hz, target_hz
    );
    println!();
    println!("Burst Timing:");
    println!("  Average:             {:.2} ms", avg_burst);
    println!("  Min:                 {:.2} ms", min_burst);
    println!("  Max:                 {:.2} ms", max_burst);
    println!("  P95:                 {:.2} ms", p95_burst);
    println!("  P99:                 {:.2} ms", p99_burst);
    println!();
    println!("Phase Breakdown:");
    println!(
        "  Injection (avg):     {:.2} ms ({:.1}%)",
        avg_inject,
        (avg_inject / avg_burst) * 100.0
    );
    println!(
        "  Execution (avg):     {:.2} ms ({:.1}%)",
        avg_exec,
        (avg_exec / avg_burst) * 100.0
    );
}

fn analyze_bottlenecks(metrics: &PerformanceMetrics, target_hz: f32) {
    let target_time = 1000.0 / target_hz;
    let avg_burst = metrics.avg(&metrics.burst_times).as_secs_f32() * 1000.0;
    let avg_exec = metrics.avg(&metrics.exec_times).as_secs_f32() * 1000.0;

    println!("Target burst time:     {:.2} ms", target_time);
    println!("Actual burst time:     {:.2} ms", avg_burst);
    println!();

    if avg_burst > target_time {
        let overage = avg_burst - target_time;
        let overage_pct = (overage / target_time) * 100.0;
        println!("⚠️  BOTTLENECK DETECTED:");
        println!(
            "  Over budget by:      {:.2} ms ({:.1}%)",
            overage, overage_pct
        );
        println!();
        println!(
            "  Execution phase uses {:.2} ms ({:.1}% of burst)",
            avg_exec,
            (avg_exec / avg_burst) * 100.0
        );
        println!();
        println!("💡 Recommendations:");
        println!("  • Run with flamegraph to identify hot paths:");
        println!("    cargo flamegraph --example profile_projection -- --with-projection");
        println!("  • Profile propagation engine");
        println!("  • Check synapse traversal overhead");
    } else {
        let headroom = target_time - avg_burst;
        println!("✅ NO BOTTLENECK DETECTED");
        println!(
            "  System has {:.2} ms headroom ({:.1}%)",
            headroom,
            (headroom / target_time) * 100.0
        );
    }
}
