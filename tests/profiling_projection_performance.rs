// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/// Profiling test for projection performance bottleneck analysis
///
/// This test creates a minimal genome with:
/// 1. Baseline: power area (128x128) → measures performance with no projections
/// 2. With projection: power → iic400 (128x128) → measures projection overhead
///
/// No external connections (ZMQ/WebSocket) - pure internal processing
///
/// Run with:
/// ```bash
/// cd feagi-core
/// cargo test --release --test profiling_projection_performance -- --nocapture
/// ```
use feagi_brain_development::{ConnectomeManager, Neuroembryogenesis};
use feagi_evolutionary::genome::map_old_id_to_new;
use feagi_evolutionary::{templates, RuntimeGenome};
use feagi_npu_burst_engine::backend::CPUBackend;
use feagi_npu_burst_engine::{DynamicNPU, TracingMutex};
use feagi_npu_runtime::StdRuntime;
use feagi_structures::genomic::cortical_area::{
    CoreCorticalType, CorticalArea, CorticalAreaDimensions as Dimensions, CorticalID,
};
use feagi_structures::genomic::descriptors::GenomeCoordinate3D;
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::{Duration, Instant};

const TEST_DURATION_BURSTS: usize = 300; // Run for 300 bursts to get stable metrics
const BURST_FREQUENCY_HZ: f32 = 15.0; // 15 Hz as requested
const AREA_SIZE: (u32, u32, u32) = (128, 128, 1); // 128x128x1 = 16,384 neurons
#[allow(dead_code)]
const STIMULUS_NEURONS: usize = 500; // Inject 500 random neurons per burst

#[test]
fn profiling_projection_performance() {
    // println!("\n{}", "=".repeat(80));
    println!("PROJECTION PERFORMANCE PROFILING TEST");
    println!("{}", "=".repeat(80));
    println!();

    // Test 1: Baseline (no projections)
    println!("🔬 TEST 1: Baseline Performance (power area only, no projections)");
    println!("{}", "-".repeat(80));
    let baseline_metrics = run_profiling_test(false);
    print_metrics("BASELINE", &baseline_metrics);

    println!();

    // Test 2: With projection
    println!("🔬 TEST 2: With Projection (power → iic400, 128x128)");
    println!("{}", "-".repeat(80));
    let projection_metrics = run_profiling_test(true);
    print_metrics("WITH PROJECTION", &projection_metrics);

    println!();

    // Comparison
    println!("{}", "=".repeat(80));
    println!("📊 PERFORMANCE COMPARISON");
    println!("{}", "=".repeat(80));
    println!();

    compare_metrics(&baseline_metrics, &projection_metrics);

    // Bottleneck analysis
    println!();
    println!("{}", "=".repeat(80));
    println!("🔍 BOTTLENECK ANALYSIS");
    println!("{}", "=".repeat(80));
    println!();

    analyze_bottlenecks(&baseline_metrics, &projection_metrics);
}

/// Run profiling test with or without projection
fn run_profiling_test(with_projection: bool) -> PerformanceMetrics {
    // Create genome
    let genome = create_test_genome(with_projection);

    let runtime = StdRuntime;
    let backend = CPUBackend::new();
    let npu = DynamicNPU::new_f32(runtime, backend, 1_000_000, 10_000_000, 10)
        .expect("Failed to create NPU");
    let npu = Arc::new(TracingMutex::new(npu, "ProfilingNPU"));
    let manager = Arc::new(RwLock::new(ConnectomeManager::new_for_testing_with_npu(
        npu.clone(),
    )));

    let mut neuro = Neuroembryogenesis::new(manager.clone());
    neuro
        .develop_from_genome(&genome)
        .expect("Failed to load genome");

    {
        let mut manager_lock = manager.write();
        manager_lock
            .ensure_core_cortical_areas()
            .expect("Failed to ensure core cortical areas");
    }

    println!("   ✓ Genome loaded");
    println!(
        "   ✓ Burst frequency: {} Hz ({:.1}ms per burst)",
        BURST_FREQUENCY_HZ,
        1000.0 / BURST_FREQUENCY_HZ
    );
    println!(
        "   ✓ Area size: {}x{}x{} = {} neurons",
        AREA_SIZE.0,
        AREA_SIZE.1,
        AREA_SIZE.2,
        AREA_SIZE.0 * AREA_SIZE.1 * AREA_SIZE.2
    );
    if with_projection {
        println!("   ✓ Projection: power00 → iic400");
    }
    println!();

    let power_id = CoreCorticalType::Power.to_cortical_id();

    // Warmup: 50 bursts
    println!("   Warming up (50 bursts)...");
    for _ in 0..50 {
        inject_power_stimulus(&npu, &power_id, STIMULUS_NEURONS);
        let npu_lock = npu.lock().expect("Failed to lock NPU");
        npu_lock.process_burst().expect("Burst failed");
    }

    // Profiling run
    println!("   Running profiling ({} bursts)...", TEST_DURATION_BURSTS);

    let mut metrics = PerformanceMetrics::new();
    let test_start = Instant::now();

    for burst_num in 0..TEST_DURATION_BURSTS {
        let burst_start = Instant::now();

        // Phase 1: Inject stimulus
        let inject_start = Instant::now();
        inject_power_stimulus(&npu, &power_id, STIMULUS_NEURONS);
        let inject_duration = inject_start.elapsed();

        // Phase 2: Run burst
        let burst_exec_start = Instant::now();
        {
            let npu_lock = npu.lock().expect("Failed to lock NPU");
            npu_lock.process_burst().expect("Burst failed");
        }
        let burst_exec_duration = burst_exec_start.elapsed();

        let burst_duration = burst_start.elapsed();

        metrics.record_burst(burst_duration, inject_duration, burst_exec_duration);

        // Log progress every 50 bursts
        if (burst_num + 1) % 50 == 0 {
            println!(
                "      Progress: {}/{} bursts ({:.1}%)",
                burst_num + 1,
                TEST_DURATION_BURSTS,
                ((burst_num + 1) as f32 / TEST_DURATION_BURSTS as f32) * 100.0
            );
        }
    }

    let test_duration = test_start.elapsed();
    metrics.total_duration = test_duration;

    println!("   ✓ Profiling complete");
    println!();

    metrics
}

/// Create test genome with power area and optionally iic400 + projection
fn create_test_genome(with_projection: bool) -> RuntimeGenome {
    let mut genome = templates::create_genome_with_core_areas(
        "profiling-test".to_string(),
        "Projection Performance Test".to_string(),
    );

    // Add core morphologies
    templates::add_core_morphologies(&mut genome.morphologies);

    // Create power00 area (replace default _power with larger one)
    let power_id = CoreCorticalType::Power.to_cortical_id();
    let power_area = CorticalArea::new(
        power_id,
        1, // cortical_idx (0 is reserved for _death, 1 for _power)
        "Power".to_string(),
        Dimensions::new(AREA_SIZE.0, AREA_SIZE.1, AREA_SIZE.2)
            .expect("Failed to create power dimensions"),
        GenomeCoordinate3D::new(0, 0, 0),
        power_id
            .as_cortical_type()
            .expect("Power cortical ID should map to Core type"),
    )
    .expect("Failed to create power area");

    genome.cortical_areas.insert(power_id, power_area);

    // If testing with projection, add iic400 and mapping
    if with_projection {
        let iic400_base64 =
            map_old_id_to_new("iic400").expect("Failed to map iic400 to new cortical ID");
        let iic400_id =
            CorticalID::try_from_base_64(&iic400_base64).expect("Invalid iic400 cortical ID");
        let iic400 = CorticalArea::new(
            iic400_id,
            3, // cortical_idx
            "Vision Input".to_string(),
            Dimensions::new(AREA_SIZE.0, AREA_SIZE.1, AREA_SIZE.2)
                .expect("Failed to create iic400 dimensions"),
            GenomeCoordinate3D::new(10, 0, 0),
            iic400_id
                .as_cortical_type()
                .expect("iic400 cortical ID should map to IO type"),
        )
        .expect("Failed to create iic400 area");

        genome.cortical_areas.insert(iic400_id, iic400);

        // Note: Projections are created during neuroembryogenesis if morphologies are defined
        // For this test, we'll rely on the BDU to establish connections
    }

    genome
}

/// Inject random stimulus into power area
fn inject_power_stimulus(
    npu: &Arc<TracingMutex<DynamicNPU>>,
    cortical_id: &CorticalID,
    neuron_count: usize,
) {
    // Create a simple fixed pattern for stimulus (avoid rand dependency)
    let max_id = (AREA_SIZE.0 * AREA_SIZE.1) as usize;
    let stride = max_id / neuron_count.max(1);

    let mut xyzp_data = Vec::with_capacity(neuron_count);
    for i in 0..neuron_count {
        let flat_id = ((i * stride) % max_id) as u32;
        let x = flat_id % AREA_SIZE.0;
        let y = flat_id / AREA_SIZE.0;
        let z = 0;
        let potential = 75.0;
        xyzp_data.push((x, y, z, potential));
    }

    let mut npu_lock = npu.lock().expect("Failed to lock NPU");
    npu_lock.inject_sensory_xyzp_by_id(cortical_id, &xyzp_data);
}

/// Performance metrics collector
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
            burst_times: Vec::with_capacity(TEST_DURATION_BURSTS),
            inject_times: Vec::with_capacity(TEST_DURATION_BURSTS),
            exec_times: Vec::with_capacity(TEST_DURATION_BURSTS),
            total_duration: Duration::ZERO,
        }
    }

    fn record_burst(&mut self, total: Duration, inject: Duration, exec: Duration) {
        self.burst_times.push(total);
        self.inject_times.push(inject);
        self.exec_times.push(exec);
    }

    fn avg_burst_time(&self) -> Duration {
        let sum: Duration = self.burst_times.iter().sum();
        sum / self.burst_times.len() as u32
    }

    fn avg_inject_time(&self) -> Duration {
        let sum: Duration = self.inject_times.iter().sum();
        sum / self.inject_times.len() as u32
    }

    fn avg_exec_time(&self) -> Duration {
        let sum: Duration = self.exec_times.iter().sum();
        sum / self.exec_times.len() as u32
    }

    fn max_burst_time(&self) -> Duration {
        *self.burst_times.iter().max().unwrap()
    }

    fn min_burst_time(&self) -> Duration {
        *self.burst_times.iter().min().unwrap()
    }

    fn p95_burst_time(&self) -> Duration {
        let mut sorted = self.burst_times.clone();
        sorted.sort();
        let idx = (sorted.len() as f32 * 0.95) as usize;
        sorted[idx]
    }

    fn p99_burst_time(&self) -> Duration {
        let mut sorted = self.burst_times.clone();
        sorted.sort();
        let idx = (sorted.len() as f32 * 0.99) as usize;
        sorted[idx]
    }

    fn achieved_hz(&self) -> f32 {
        let bursts = self.burst_times.len() as f32;
        let total_secs = self.total_duration.as_secs_f32();
        bursts / total_secs
    }
}

/// Print metrics table
fn print_metrics(label: &str, metrics: &PerformanceMetrics) {
    println!("   Metrics for: {}", label);
    println!("   {}", "-".repeat(76));
    println!(
        "   Total duration:        {:>10.2} s",
        metrics.total_duration.as_secs_f32()
    );
    println!(
        "   Total bursts:          {:>10}",
        metrics.burst_times.len()
    );
    println!(
        "   Achieved frequency:    {:>10.2} Hz (target: {} Hz)",
        metrics.achieved_hz(),
        BURST_FREQUENCY_HZ
    );
    println!();
    println!("   Burst Timing:");
    println!(
        "     Average:             {:>10.2} ms",
        metrics.avg_burst_time().as_secs_f32() * 1000.0
    );
    println!(
        "     Min:                 {:>10.2} ms",
        metrics.min_burst_time().as_secs_f32() * 1000.0
    );
    println!(
        "     Max:                 {:>10.2} ms",
        metrics.max_burst_time().as_secs_f32() * 1000.0
    );
    println!(
        "     P95:                 {:>10.2} ms",
        metrics.p95_burst_time().as_secs_f32() * 1000.0
    );
    println!(
        "     P99:                 {:>10.2} ms",
        metrics.p99_burst_time().as_secs_f32() * 1000.0
    );
    println!();
    println!("   Phase Breakdown:");
    println!(
        "     Injection (avg):     {:>10.2} ms ({:>5.1}%)",
        metrics.avg_inject_time().as_secs_f32() * 1000.0,
        (metrics.avg_inject_time().as_secs_f32() / metrics.avg_burst_time().as_secs_f32()) * 100.0
    );
    println!(
        "     Execution (avg):     {:>10.2} ms ({:>5.1}%)",
        metrics.avg_exec_time().as_secs_f32() * 1000.0,
        (metrics.avg_exec_time().as_secs_f32() / metrics.avg_burst_time().as_secs_f32()) * 100.0
    );
}

/// Compare two metric sets
fn compare_metrics(baseline: &PerformanceMetrics, projection: &PerformanceMetrics) {
    let baseline_avg = baseline.avg_burst_time().as_secs_f32() * 1000.0;
    let projection_avg = projection.avg_burst_time().as_secs_f32() * 1000.0;
    let slowdown = projection_avg / baseline_avg;
    let overhead_ms = projection_avg - baseline_avg;
    let overhead_pct = (slowdown - 1.0) * 100.0;

    println!("   Burst Time Comparison:");
    println!("     Baseline:            {:>10.2} ms", baseline_avg);
    println!("     With Projection:     {:>10.2} ms", projection_avg);
    println!(
        "     Overhead:            {:>10.2} ms (+{:.1}%)",
        overhead_ms, overhead_pct
    );
    println!("     Slowdown Factor:     {:>10.2}x", slowdown);
    println!();

    let baseline_hz = baseline.achieved_hz();
    let projection_hz = projection.achieved_hz();

    println!("   Frequency Comparison:");
    println!("     Baseline:            {:>10.2} Hz", baseline_hz);
    println!("     With Projection:     {:>10.2} Hz", projection_hz);
    println!(
        "     Loss:                {:>10.2} Hz ({:.1}%)",
        baseline_hz - projection_hz,
        ((baseline_hz - projection_hz) / baseline_hz) * 100.0
    );
    println!();

    // Phase comparison
    let baseline_inject = baseline.avg_inject_time().as_secs_f32() * 1000.0;
    let projection_inject = projection.avg_inject_time().as_secs_f32() * 1000.0;
    let inject_overhead = projection_inject - baseline_inject;

    let baseline_exec = baseline.avg_exec_time().as_secs_f32() * 1000.0;
    let projection_exec = projection.avg_exec_time().as_secs_f32() * 1000.0;
    let exec_overhead = projection_exec - baseline_exec;

    println!("   Phase Overhead Breakdown:");
    println!(
        "     Injection phase:     {:>10.2} ms → {:>10.2} ms (+{:.2} ms)",
        baseline_inject, projection_inject, inject_overhead
    );
    println!(
        "     Execution phase:     {:>10.2} ms → {:>10.2} ms (+{:.2} ms)",
        baseline_exec, projection_exec, exec_overhead
    );

    // P95/P99 comparison (tail latency is important!)
    let baseline_p95 = baseline.p95_burst_time().as_secs_f32() * 1000.0;
    let projection_p95 = projection.p95_burst_time().as_secs_f32() * 1000.0;
    let baseline_p99 = baseline.p99_burst_time().as_secs_f32() * 1000.0;
    let projection_p99 = projection.p99_burst_time().as_secs_f32() * 1000.0;

    println!();
    println!("   Tail Latency (P95/P99):");
    println!("     Baseline P95:        {:>10.2} ms", baseline_p95);
    println!(
        "     Projection P95:      {:>10.2} ms (+{:.2} ms)",
        projection_p95,
        projection_p95 - baseline_p95
    );
    println!("     Baseline P99:        {:>10.2} ms", baseline_p99);
    println!(
        "     Projection P99:      {:>10.2} ms (+{:.2} ms)",
        projection_p99,
        projection_p99 - baseline_p99
    );
}

/// Analyze bottlenecks based on metrics
fn analyze_bottlenecks(baseline: &PerformanceMetrics, projection: &PerformanceMetrics) {
    let baseline_exec = baseline.avg_exec_time().as_secs_f32() * 1000.0;
    let projection_exec = projection.avg_exec_time().as_secs_f32() * 1000.0;
    let exec_overhead = projection_exec - baseline_exec;

    let baseline_inject = baseline.avg_inject_time().as_secs_f32() * 1000.0;
    let projection_inject = projection.avg_inject_time().as_secs_f32() * 1000.0;
    let inject_overhead = projection_inject - baseline_inject;

    let target_burst_time = 1000.0 / BURST_FREQUENCY_HZ; // ms
    let projection_burst_time = projection.avg_burst_time().as_secs_f32() * 1000.0;

    println!(
        "   Target burst time:     {:.2} ms (for {} Hz)",
        target_burst_time, BURST_FREQUENCY_HZ
    );
    println!("   Actual burst time:     {:.2} ms", projection_burst_time);
    println!();

    if projection_burst_time > target_burst_time {
        let time_over = projection_burst_time - target_burst_time;
        println!(
            "   ⚠️  BOTTLENECK DETECTED: {:.2} ms over budget ({:.1}%)",
            time_over,
            (time_over / target_burst_time) * 100.0
        );
        println!();

        // Identify primary bottleneck
        if exec_overhead > 5.0 {
            println!("   🔍 PRIMARY BOTTLENECK: Burst Execution Phase");
            println!(
                "      - Execution overhead: +{:.2} ms ({:.1}%)",
                exec_overhead,
                (exec_overhead / baseline_exec) * 100.0
            );
            println!("      - Likely causes:");
            println!(
                "        • Projection/propagation processing ({}K neurons)",
                (AREA_SIZE.0 * AREA_SIZE.1) / 1000
            );
            println!("        • Synapse traversal overhead");
            println!("        • Fire queue management");
            println!("        • Memory access patterns");
            println!();
            println!("   💡 Recommended actions:");
            println!(
                "      1. Profile propagation engine with {}x{} projection",
                AREA_SIZE.0, AREA_SIZE.1
            );
            println!("      2. Check synapse array access patterns (cache misses?)");
            println!("      3. Measure fire queue contention");
            println!("      4. Consider batch propagation optimizations");
            println!("      5. Use 'perf' or 'flamegraph' to identify hot paths:");
            println!("         cargo flamegraph --test profiling_projection_performance -- --ignored --nocapture");
        } else if inject_overhead > 5.0 {
            println!("   🔍 PRIMARY BOTTLENECK: Injection Phase");
            println!(
                "      - Injection overhead: +{:.2} ms ({:.1}%)",
                inject_overhead,
                (inject_overhead / baseline_inject) * 100.0
            );
            println!("      - Likely causes:");
            println!("        • XYZP coordinate-to-ID conversion");
            println!("        • NPU lock contention");
            println!("        • FCL queue operations");
            println!();
            println!("   💡 Recommended actions:");
            println!("      1. Profile inject_sensory_xyzp() method");
            println!("      2. Consider fine-grained locking");
            println!("      3. Pre-allocate injection buffers");
        } else {
            println!("   🔍 Overhead is distributed across multiple phases");
            println!("      - No single dominant bottleneck");
            println!("      - May need system-wide optimization");
        }
    } else {
        println!(
            "   ✅ NO BOTTLENECK: System can sustain {} Hz with projection",
            BURST_FREQUENCY_HZ
        );
        println!(
            "      - Projection overhead: +{:.2} ms ({:.1}%)",
            exec_overhead,
            (exec_overhead / baseline_exec) * 100.0
        );
        println!(
            "      - System has {:.2} ms headroom",
            target_burst_time - projection_burst_time
        );
    }
}
