// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

#[cfg(all(test, feature = "cuda"))]
mod cuda_realistic_genome_tests {
    use feagi_burst_engine::backend::{GpuConfig, is_cuda_available};
    use feagi_burst_engine::RustNPU;
    use feagi_runtime_std::{NeuronArray, SynapseArray};

    /// Helper to create a realistic FEAGI genome scenario
    fn create_realistic_genome() -> (NeuronArray<f32>, SynapseArray) {
        // Neuron counts
        let power_neurons = 1;
        let area_a_neurons = 512 * 512; // 262,144
        let area_b_neurons = 1024 * 1024; // 1,048,576
        let area_c_neurons = 1;
        let total_neurons = power_neurons + area_a_neurons + area_b_neurons + area_c_neurons;
        
        println!("🧬 Creating realistic FEAGI genome:");
        println!("   _power: {} neurons", power_neurons);
        println!("   Area A: {} neurons (512x512, 5% excitability)", area_a_neurons);
        println!("   Area B: {} neurons (1024x1024)", area_b_neurons);
        println!("   Area C: {} neurons (gauge)", area_c_neurons);
        println!("   Total:  {} neurons", total_neurons);
        
        let mut neuron_array = NeuronArray::new(total_neurons);
        
        // CRITICAL: Mark all neurons as valid (SIMD-optimized)
        neuron_array.valid_mask[..total_neurons].fill(true);
        
        // Neuron ID ranges
        let power_start = 0;
        let power_end = power_start + power_neurons;
        let a_start = power_end;
        let a_end = a_start + area_a_neurons;
        let b_start = a_end;
        let b_end = b_start + area_b_neurons;
        let c_start = b_end;
        let c_end = c_start + area_c_neurons;
        
        // Initialize _power area (neuron 0)
        // CRITICAL: Power neuron auto-fires via FEAGI's power injection system
        // NO refractory, NO snooze, NO leak, threshold = 1.0
        for i in power_start..power_end {
            neuron_array.membrane_potentials[i] = 0.0;
            neuron_array.thresholds[i] = 1.0; // THRESHOLD = 1.0
            neuron_array.leak_coefficients[i] = 0.0; // NO LEAK
            neuron_array.resting_potentials[i] = 0.0;
            neuron_array.excitabilities[i] = 1.0;
            neuron_array.refractory_countdowns[i] = 0;
            neuron_array.refractory_periods[i] = 0; // NO REFRACTORY
            neuron_array.cortical_areas[i] = 1; // cortical_area=1 marks power neurons
        }
        
        // Initialize Area A with 5% excitability
        // NO refractory, NO snooze, NO leak
        for i in a_start..a_end {
            neuron_array.membrane_potentials[i] = 0.0;
            neuron_array.thresholds[i] = 1.0;
            neuron_array.leak_coefficients[i] = 0.0; // NO LEAK
            neuron_array.resting_potentials[i] = 0.0;
            neuron_array.excitabilities[i] = 0.05; // 5% excitability
            neuron_array.refractory_countdowns[i] = 0;
            neuron_array.refractory_periods[i] = 0; // NO REFRACTORY
            neuron_array.cortical_areas[i] = 2;
        }
        
        // Initialize Area B
        // NO refractory, NO snooze, NO leak
        for i in b_start..b_end {
            neuron_array.membrane_potentials[i] = 0.0;
            neuron_array.thresholds[i] = 1.0;
            neuron_array.leak_coefficients[i] = 0.0; // NO LEAK
            neuron_array.resting_potentials[i] = 0.0;
            neuron_array.excitabilities[i] = 0.05;
            neuron_array.refractory_countdowns[i] = 0;
            neuron_array.refractory_periods[i] = 0; // NO REFRACTORY
            neuron_array.cortical_areas[i] = 3;
        }
        
        // Initialize Area C (gauge)
        // NO refractory, NO snooze, NO leak
        for i in c_start..c_end {
            neuron_array.membrane_potentials[i] = 0.0;
            neuron_array.thresholds[i] = 1.0;
            neuron_array.leak_coefficients[i] = 0.0; // NO LEAK
            neuron_array.resting_potentials[i] = 0.0;
            neuron_array.excitabilities[i] = 0.0; // Never fires (gauge only)
            neuron_array.refractory_countdowns[i] = 0;
            neuron_array.refractory_periods[i] = 0; // NO REFRACTORY
            neuron_array.cortical_areas[i] = 4;
        }
        
        // CRITICAL: Set neuron count (was missing!)
        neuron_array.count = total_neurons;
        
        // Create synapses
        let synapse_count = 1000 + 10000 + 1000;
        let mut synapse_array = SynapseArray::new(synapse_count);
        
        let mut idx = 0;
        
        println!("   Creating synaptic connections:");
        
        // _power → Area A (PSP UNIFORMITY ENABLED)
        // With PSP uniformity: Each synapse gets full cortical area PSP value (not divided)
        // Power area PSP = 5 → all synapses get PSP=5
        let power_psp = 5;
        for target_offset in 0..1000 {
            if idx >= synapse_count { break; }
            let target = a_start + (target_offset * 262) % area_a_neurons;
            synapse_array.source_neurons[idx] = power_start as u32;
            synapse_array.target_neurons[idx] = target as u32;
            synapse_array.weights[idx] = 10;
            synapse_array.postsynaptic_potentials[idx] = power_psp; // Full PSP value (uniformity)
            synapse_array.types[idx] = 0;
            idx += 1;
        }
        println!("   _power → A: {} synapses (PSP uniformity: PSP={} per synapse)", 1000, power_psp);
        
        // Area A → Area B (PSP UNIFORMITY ENABLED)
        // Area A PSP = 1 → all synapses get PSP=1
        let area_a_psp = 1;
        for src_offset in 0..100 {
            let src = a_start + (src_offset * 2621) % area_a_neurons;
            for dst_offset in 0..100 {
                if idx >= synapse_count { break; }
                let dst = b_start + (src_offset * 10000 + dst_offset * 100) % area_b_neurons;
                synapse_array.source_neurons[idx] = src as u32;
                synapse_array.target_neurons[idx] = dst as u32;
                synapse_array.weights[idx] = 10;
                synapse_array.postsynaptic_potentials[idx] = area_a_psp; // Full PSP value (uniformity)
                synapse_array.types[idx] = 0;
                idx += 1;
            }
        }
        println!("   A → B: {} synapses (PSP uniformity: PSP={} per synapse)", 10000, area_a_psp);
        
        // Area B → Area C (PSP UNIFORMITY ENABLED)
        // Area B PSP = 1 → all synapses get PSP=1
        let area_b_psp = 1;
        for src_offset in 0..1000 {
            if idx >= synapse_count { break; }
            let src = b_start + (src_offset * 1048) % area_b_neurons;
            synapse_array.source_neurons[idx] = src as u32;
            synapse_array.target_neurons[idx] = c_start as u32;
            synapse_array.weights[idx] = 10;
            synapse_array.postsynaptic_potentials[idx] = area_b_psp; // Full PSP value (uniformity)
            synapse_array.types[idx] = 0;
            idx += 1;
        }
        println!("   B → C: {} synapses (PSP uniformity: PSP={} per synapse)", 1000, area_b_psp);
        
        synapse_array.count = idx;
        println!("   Total synapses: {}", idx);
        
        // CRITICAL: Mark synapses as valid and rebuild source_index (SIMD-optimized)
        synapse_array.valid_mask[..synapse_array.count].fill(true);
        
        synapse_array.source_index.clear();
        for syn_idx in 0..synapse_array.count {
            if synapse_array.valid_mask[syn_idx] {
                let source_id = synapse_array.source_neurons[syn_idx];
                synapse_array.source_index
                    .entry(source_id)
                    .or_insert_with(Vec::new)
                    .push(syn_idx);
            }
        }
        println!("   Rebuilt source_index for {} source neurons", synapse_array.source_index.len());
        
        (neuron_array, synapse_array)
    }
    
    #[test]
    #[ignore] // Requires actual CUDA hardware
    fn test_realistic_genome_continuous_simulation() {
        if !is_cuda_available() {
            println!("⚠️  CUDA not available, skipping test");
            return;
        }
        
        println!("\n🧪 Testing realistic genome: Continuous simulation (1 min @ 30Hz)");
        
        let (neuron_array, synapse_array) = create_realistic_genome();
        
        // Test parameters
        let target_frequency_hz = 30.0;
        let simulation_duration_sec = 60.0;
        let total_bursts = (target_frequency_hz * simulation_duration_sec) as usize; // 1800 bursts
        
        println!("   Target frequency: {} Hz", target_frequency_hz);
        println!("   Duration: {} seconds", simulation_duration_sec);
        println!("   Total bursts: {}", total_bursts);
        
        // Create NPUs using FEAGI's high-level API
        let mut cpu_npu = RustNPU::<f32>::new_cpu_only(
            neuron_array.count,
            synapse_array.count,
            100, // fire_ledger_window
        );
        
        let mut cuda_npu = RustNPU::<f32>::new(
            neuron_array.count,
            synapse_array.count,
            100, // fire_ledger_window
            Some(&GpuConfig {
                use_gpu: true,
                hybrid_enabled: false,
                gpu_threshold: 1, // Force GPU for any size
                gpu_memory_fraction: 0.9,
            }),
        );
        
        // Populate both NPUs with the same genome using batch API
        // Extract neuron data as vectors
        let thresholds = neuron_array.thresholds[..neuron_array.count].to_vec();
        let leak_coefficients = neuron_array.leak_coefficients[..neuron_array.count].to_vec();
        let resting_potentials = neuron_array.resting_potentials[..neuron_array.count].to_vec();
        let neuron_types = vec![0i32; neuron_array.count];
        let refractory_periods = neuron_array.refractory_periods[..neuron_array.count].to_vec();
        let excitabilities = neuron_array.excitabilities[..neuron_array.count].to_vec();
        let consecutive_fire_limits = vec![0u16; neuron_array.count];
        let snooze_periods = vec![0u16; neuron_array.count];
        let mp_charge_accumulations = vec![true; neuron_array.count];
        let cortical_areas = neuron_array.cortical_areas[..neuron_array.count].to_vec();
        let x_coords: Vec<u32> = (0..neuron_array.count).map(|i| neuron_array.coordinates[i * 3]).collect();
        let y_coords: Vec<u32> = (0..neuron_array.count).map(|i| neuron_array.coordinates[i * 3 + 1]).collect();
        let z_coords: Vec<u32> = (0..neuron_array.count).map(|i| neuron_array.coordinates[i * 3 + 2]).collect();
        
        // Add neurons to CPU NPU
        let (cpu_neuron_count, _) = cpu_npu.add_neurons_batch(
            thresholds.clone(),
            leak_coefficients.clone(),
            resting_potentials.clone(),
            neuron_types.clone(),
            refractory_periods.clone(),
            excitabilities.clone(),
            consecutive_fire_limits.clone(),
            snooze_periods.clone(),
            mp_charge_accumulations.clone(),
            cortical_areas.clone(),
            x_coords.clone(),
            y_coords.clone(),
            z_coords.clone(),
        );
        println!("   Added {} neurons to CPU NPU", cpu_neuron_count);
        
        // Add neurons to CUDA NPU
        let (cuda_neuron_count, _) = cuda_npu.add_neurons_batch(
            thresholds,
            leak_coefficients,
            resting_potentials,
            neuron_types,
            refractory_periods,
            excitabilities,
            consecutive_fire_limits,
            snooze_periods,
            mp_charge_accumulations,
            cortical_areas,
            x_coords,
            y_coords,
            z_coords,
        );
        println!("   Added {} neurons to CUDA NPU", cuda_neuron_count);
        
        // Extract synapse data as vectors
        let source_neurons: Vec<feagi_types::NeuronId> = synapse_array.source_neurons[..synapse_array.count]
            .iter().map(|&id| feagi_types::NeuronId(id)).collect();
        let target_neurons: Vec<feagi_types::NeuronId> = synapse_array.target_neurons[..synapse_array.count]
            .iter().map(|&id| feagi_types::NeuronId(id)).collect();
        let weights: Vec<feagi_types::SynapticWeight> = synapse_array.weights[..synapse_array.count]
            .iter().map(|&w| feagi_types::SynapticWeight(w)).collect();
        let psps: Vec<feagi_types::SynapticConductance> = synapse_array.postsynaptic_potentials[..synapse_array.count]
            .iter().map(|&psp| feagi_types::SynapticConductance(psp)).collect();
        let synapse_types: Vec<feagi_types::SynapseType> = synapse_array.types[..synapse_array.count]
            .iter().map(|&t| if t == 0 { feagi_types::SynapseType::Excitatory } else { feagi_types::SynapseType::Inhibitory }).collect();
        
        // Add synapses to CPU NPU
        let (cpu_synapse_count, _) = cpu_npu.add_synapses_batch(
            source_neurons.clone(),
            target_neurons.clone(),
            weights.clone(),
            psps.clone(),
            synapse_types.clone(),
        );
        println!("   Added {} synapses to CPU NPU", cpu_synapse_count);
        
        // Add synapses to CUDA NPU
        let (cuda_synapse_count, _) = cuda_npu.add_synapses_batch(
            source_neurons,
            target_neurons,
            weights,
            psps,
            synapse_types,
        );
        println!("   Added {} synapses to CUDA NPU", cuda_synapse_count);
        
        // Track metrics
        let mut cpu_total_fired = 0usize;
        let mut gpu_total_fired = 0usize;
        let mut mismatches = 0usize;
        let mut cpu_burst_times = Vec::with_capacity(total_bursts);
        let mut gpu_burst_times = Vec::with_capacity(total_bursts);
        
        println!("\n🚀 Starting continuous simulation...");
        println!("   Power neurons (cortical_area=1) will auto-inject via FEAGI's burst engine");
        let sim_start = std::time::Instant::now();
        
        for burst_idx in 0..total_bursts {
            // CPU burst - FEAGI handles everything internally
            let cpu_burst_start = std::time::Instant::now();
            let cpu_result = cpu_npu.process_burst()
                .expect("CPU burst failed");
            let cpu_burst_time = cpu_burst_start.elapsed();
            cpu_burst_times.push(cpu_burst_time);
            cpu_total_fired += cpu_result.neuron_count;
            
            // GPU burst - FEAGI handles everything internally
            let gpu_burst_start = std::time::Instant::now();
            let gpu_result = cuda_npu.process_burst()
                .expect("GPU burst failed");
            let gpu_burst_time = gpu_burst_start.elapsed();
            gpu_burst_times.push(gpu_burst_time);
            gpu_total_fired += gpu_result.neuron_count;
            
            // Compare results
            if cpu_result.neuron_count != gpu_result.neuron_count {
                mismatches += 1;
                if mismatches <= 10 {
                    println!("⚠️  Burst {}: Fire count mismatch! CPU={}, GPU={}", 
                        burst_idx, cpu_result.neuron_count, gpu_result.neuron_count);
                }
            }
            
            // Progress reporting every 300 bursts (every 10 seconds @ 30Hz)
            if (burst_idx + 1) % 300 == 0 {
                let elapsed = sim_start.elapsed().as_secs_f32();
                let actual_hz = (burst_idx + 1) as f32 / elapsed;
                let cpu_avg_ms = cpu_burst_times.iter().sum::<std::time::Duration>().as_secs_f64() * 1000.0 / cpu_burst_times.len() as f64;
                let gpu_avg_ms = gpu_burst_times.iter().sum::<std::time::Duration>().as_secs_f64() * 1000.0 / gpu_burst_times.len() as f64;
                println!("   [{:.1}s] Burst {}/{}: {:.1} Hz actual | CPU {:.2}ms | GPU {:.2}ms | Mismatches: {}", 
                    elapsed, burst_idx + 1, total_bursts, actual_hz, cpu_avg_ms, gpu_avg_ms, mismatches);
            }
        }
        
        let total_time = sim_start.elapsed();
        let actual_frequency = total_bursts as f32 / total_time.as_secs_f32();
        
        // Calculate statistics
        let cpu_avg_burst_time = cpu_burst_times.iter().sum::<std::time::Duration>() / cpu_burst_times.len() as u32;
        let gpu_avg_burst_time = gpu_burst_times.iter().sum::<std::time::Duration>() / gpu_burst_times.len() as u32;
        
        let cpu_min = cpu_burst_times.iter().min().unwrap();
        let cpu_max = cpu_burst_times.iter().max().unwrap();
        let gpu_min = gpu_burst_times.iter().min().unwrap();
        let gpu_max = gpu_burst_times.iter().max().unwrap();
        
        println!("\n📊 Simulation Results:");
        println!("   Total time: {:.2}s", total_time.as_secs_f32());
        println!("   Target frequency: {} Hz", target_frequency_hz);
        println!("   Actual frequency: {:.2} Hz", actual_frequency);
        println!("\n   CPU Performance:");
        println!("      Total neurons fired: {}", cpu_total_fired);
        println!("      Avg burst time: {:?}", cpu_avg_burst_time);
        println!("      Min/Max: {:?} / {:?}", cpu_min, cpu_max);
        println!("\n   GPU Performance:");
        println!("      Total neurons fired: {}", gpu_total_fired);
        println!("      Avg burst time: {:?}", gpu_avg_burst_time);
        println!("      Min/Max: {:?} / {:?}", gpu_min, gpu_max);
        println!("\n   Speedup: {:.2}x", 
            cpu_avg_burst_time.as_secs_f64() / gpu_avg_burst_time.as_secs_f64());
        println!("   Mismatches: {} / {} bursts ({:.2}%)", 
            mismatches, total_bursts, (mismatches as f32 / total_bursts as f32) * 100.0);
        
        // Validation
        assert_eq!(mismatches, 0, "CPU and GPU results diverged in {} bursts!", mismatches);
        
        println!("\n✅ Continuous simulation passed!");
    }
}
