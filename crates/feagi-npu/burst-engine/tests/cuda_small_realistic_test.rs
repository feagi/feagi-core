// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

#[cfg(all(test, feature = "cuda"))]
mod cuda_small_realistic_tests {
    use feagi_npu_burst_engine::backend::{is_cuda_available, GpuConfig};
    use feagi_npu_burst_engine::RustNPU;
    use feagi_npu_neural::types::{NeuronId, SynapticPsp, SynapticWeight};
    use feagi_npu_neural::SynapseType;

    /// Create a small realistic network for fast testing
    fn create_small_network_cpu(
    ) -> RustNPU<feagi_npu_runtime::StdRuntime, f32, feagi_npu_burst_engine::backend::CPUBackend>
    {
        use feagi_npu_burst_engine::backend::CPUBackend;
        use feagi_npu_runtime::StdRuntime;
        let runtime = StdRuntime;
        let backend = CPUBackend::new();
        let mut npu = RustNPU::new(runtime, backend, 1000, 10000, 100).unwrap();

        // Power neuron (cortical_area=1) - 1 neuron
        npu.add_neuron(0.5, 0.0, 0.0, 0.0, 0, 1, 1.0, 0, 0, true, 1, 0, 0, 0)
            .unwrap();

        // Area A (cortical_area=2) - 100 neurons with 5% excitability
        for i in 0..100 {
            npu.add_neuron(1.0, 0.0, 0.1, 0.0, 0, 3, 0.05, 0, 0, true, 2, i, 0, 0)
                .unwrap();
        }

        // Area B (cortical_area=3) - 100 neurons
        for i in 0..100 {
            npu.add_neuron(1.0, 0.0, 0.1, 0.0, 0, 3, 0.05, 0, 0, true, 3, i, 0, 0)
                .unwrap();
        }

        // Area C gauge (cortical_area=4) - 1 neuron
        npu.add_neuron(1.0, 0.0, 0.0, 0.0, 0, 0, 0.0, 0, 0, true, 4, 0, 0, 0)
            .unwrap();

        // Power → Area A synapses (50 connections)
        for target_offset in 0..50 {
            npu.add_synapse(
                NeuronId(0),                 // power neuron
                NeuronId(1 + target_offset), // Area A neurons
                SynapticWeight(10),
                SynapticPsp(5),
                SynapseType::Excitatory,
            )
            .unwrap();
        }

        // Area A → Area B synapses (100 connections)
        for src_offset in 0..10 {
            for dst_offset in 0..10 {
                npu.add_synapse(
                    NeuronId(1 + src_offset),                     // Area A
                    NeuronId(101 + dst_offset * 10 + src_offset), // Area B
                    SynapticWeight(10),
                    SynapticPsp(1),
                    SynapseType::Excitatory,
                )
                .unwrap();
            }
        }

        // Area B → Area C synapses (50 connections)
        for src_offset in 0..50 {
            npu.add_synapse(
                NeuronId(101 + src_offset), // Area B
                NeuronId(201),              // Area C gauge
                SynapticWeight(1),
                SynapticPsp(1),
                SynapseType::Excitatory,
            )
            .unwrap();
        }

        npu
    }

    fn create_small_network_cuda() -> RustNPU<StdRuntime, f32, CPUBackend> {
        use feagi_npu_burst_engine::backend::CPUBackend;
        use feagi_npu_runtime::StdRuntime;

        let runtime = StdRuntime;
        let backend = CPUBackend::new();
        let mut npu = RustNPU::new(runtime, backend, 1000, 10000, 100).unwrap();

        // Same network as CPU
        npu.add_neuron(0.5, 0.0, 0.0, 0.0, 0, 1, 1.0, 0, 0, true, 1, 0, 0, 0)
            .unwrap();
        for i in 0..100 {
            npu.add_neuron(1.0, 0.0, 0.1, 0.0, 0, 3, 0.05, 0, 0, true, 2, i, 0, 0)
                .unwrap();
        }
        for i in 0..100 {
            npu.add_neuron(1.0, 0.0, 0.1, 0.0, 0, 3, 0.05, 0, 0, true, 3, i, 0, 0)
                .unwrap();
        }
        npu.add_neuron(1.0, 0.0, 0.0, 0.0, 0, 0, 0.0, 0, 0, true, 4, 0, 0, 0)
            .unwrap();

        for target_offset in 0..50 {
            npu.add_synapse(
                NeuronId(0),
                NeuronId(1 + target_offset),
                SynapticWeight(10),
                SynapticPsp(5),
                SynapseType::Excitatory,
            )
            .unwrap();
        }

        for src_offset in 0..10 {
            for dst_offset in 0..10 {
                npu.add_synapse(
                    NeuronId(1 + src_offset),
                    NeuronId(101 + dst_offset * 10 + src_offset),
                    SynapticWeight(10),
                    SynapticPsp(1),
                    SynapseType::Excitatory,
                )
                .unwrap();
            }
        }

        for src_offset in 0..50 {
            npu.add_synapse(
                NeuronId(101 + src_offset),
                NeuronId(201),
                SynapticWeight(1),
                SynapticPsp(1),
                SynapseType::Excitatory,
            )
            .unwrap();
        }

        npu
    }

    #[test]
    #[ignore] // Requires actual CUDA hardware
    fn test_small_realistic_continuous_simulation() {
        if !is_cuda_available() {
            println!("⚠️  CUDA not available, skipping test");
            return;
        }

        println!("\n🧪 Testing small realistic genome: Continuous simulation (1 min @ 30Hz)");

        let cpu_npu = create_small_network_cpu();
        let cuda_npu = create_small_network_cuda();

        let target_frequency_hz = 30.0;
        let simulation_duration_sec = 60.0;
        let total_bursts = (target_frequency_hz * simulation_duration_sec) as usize;

        println!("   Small network: 202 neurons (1 power, 100 A, 100 B, 1 C gauge)");
        println!(
            "   Target: {} bursts @ {} Hz",
            total_bursts, target_frequency_hz
        );

        let mut cpu_total_fired = 0usize;
        let mut gpu_total_fired = 0usize;
        let mut mismatches = 0usize;
        let mut cpu_burst_times = Vec::with_capacity(total_bursts);
        let mut gpu_burst_times = Vec::with_capacity(total_bursts);

        println!("\n🚀 Starting continuous simulation...");
        let sim_start = std::time::Instant::now();

        for burst_idx in 0..total_bursts {
            // CPU burst
            let cpu_burst_start = std::time::Instant::now();
            let cpu_result = cpu_npu.process_burst().expect("CPU burst failed");
            let cpu_burst_time = cpu_burst_start.elapsed();
            cpu_burst_times.push(cpu_burst_time);
            cpu_total_fired += cpu_result.neuron_count;

            // GPU burst
            let gpu_burst_start = std::time::Instant::now();
            let gpu_result = cuda_npu.process_burst().expect("GPU burst failed");
            let gpu_burst_time = gpu_burst_start.elapsed();
            gpu_burst_times.push(gpu_burst_time);
            gpu_total_fired += gpu_result.neuron_count;

            if cpu_result.neuron_count != gpu_result.neuron_count {
                mismatches += 1;
                if mismatches <= 10 {
                    println!(
                        "⚠️  Burst {}: CPU={}, GPU={}",
                        burst_idx, cpu_result.neuron_count, gpu_result.neuron_count
                    );
                }
            }

            if (burst_idx + 1) % 300 == 0 {
                let elapsed = sim_start.elapsed().as_secs_f32();
                let actual_hz = (burst_idx + 1) as f32 / elapsed;
                let cpu_avg_ms = cpu_burst_times
                    .iter()
                    .sum::<std::time::Duration>()
                    .as_secs_f64()
                    * 1000.0
                    / cpu_burst_times.len() as f64;
                let gpu_avg_ms = gpu_burst_times
                    .iter()
                    .sum::<std::time::Duration>()
                    .as_secs_f64()
                    * 1000.0
                    / gpu_burst_times.len() as f64;
                println!("   [{:.1}s] Burst {}/{}: {:.1} Hz | CPU {:.3}ms | GPU {:.3}ms | Mismatches: {}",
                    elapsed, burst_idx + 1, total_bursts, actual_hz, cpu_avg_ms, gpu_avg_ms, mismatches);
            }
        }

        let total_time = sim_start.elapsed();
        let actual_frequency = total_bursts as f32 / total_time.as_secs_f32();

        let cpu_avg =
            cpu_burst_times.iter().sum::<std::time::Duration>() / cpu_burst_times.len() as u32;
        let gpu_avg =
            gpu_burst_times.iter().sum::<std::time::Duration>() / gpu_burst_times.len() as u32;

        println!("\n📊 Simulation Results:");
        println!("   Total time: {:.2}s", total_time.as_secs_f32());
        println!(
            "   Target: {} Hz, Actual: {:.2} Hz",
            target_frequency_hz, actual_frequency
        );
        println!(
            "   CPU: {} neurons fired, avg {:?}",
            cpu_total_fired, cpu_avg
        );
        println!(
            "   GPU: {} neurons fired, avg {:?}",
            gpu_total_fired, gpu_avg
        );
        println!(
            "   Speedup: {:.2}x",
            cpu_avg.as_secs_f64() / gpu_avg.as_secs_f64()
        );
        println!(
            "   Mismatches: {} / {} ({:.2}%)",
            mismatches,
            total_bursts,
            (mismatches as f32 / total_bursts as f32) * 100.0
        );

        assert_eq!(mismatches, 0, "CPU and GPU diverged!");
        assert!(cpu_total_fired > 0, "No neurons fired!");

        println!("\n✅ Continuous simulation passed!");
    }
}
