// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*
 * CUDA Backend Correctness Validation
 *
 * Compares CUDA GPU results against CPU backend to ensure numerical correctness.
 * This is the critical test for Phase 1 completion.
 */

#[cfg(all(test, feature = "cuda"))]
mod cuda_correctness_tests {
    use feagi_npu_burst_engine::backend::{
        is_cuda_available, CPUBackend, CUDABackend, ComputeBackend,
    };
    use feagi_npu_burst_engine::FireCandidateList;
    use feagi_npu_neural::types::NeuronId;
    use feagi_npu_runtime::{StdNeuronArray as NeuronArray, StdSynapseArray as SynapseArray};

    /// Helper to create a small deterministic test genome
    fn create_test_genome(
        neuron_count: usize,
        synapse_count: usize,
    ) -> (NeuronArray<f32>, SynapseArray) {
        let mut neuron_array = NeuronArray::new(neuron_count);

        // Initialize with deterministic values
        for i in 0..neuron_count {
            neuron_array.membrane_potentials[i] = 0.0;
            neuron_array.thresholds[i] = 1.0; // Fire at potential >= 1.0
            neuron_array.leak_coefficients[i] = 0.1;
            neuron_array.resting_potentials[i] = 0.0;
            neuron_array.excitabilities[i] = 1.0; // Always fire when threshold reached
            neuron_array.refractory_countdowns[i] = 0;
            neuron_array.refractory_periods[i] = 3;
        }
        neuron_array.count = neuron_count;

        let mut synapse_array = SynapseArray::new(synapse_count);

        // Create a simple connectivity pattern:
        // Neuron 0 → Neuron 1, 2, 3 (excitatory, weight=10, psp=5)
        // Neuron 1 → Neuron 4 (excitatory, weight=10, psp=5)
        // Neuron 2 → Neuron 5 (inhibitory, weight=10, psp=5)
        let mut idx = 0;

        // Neuron 0 connections
        for target in 1..4 {
            if idx < synapse_count {
                synapse_array.source_neurons[idx] = 0;
                synapse_array.target_neurons[idx] = target as u32;
                synapse_array.weights[idx] = 10;
                synapse_array.postsynaptic_potentials[idx] = 5;
                synapse_array.types[idx] = 0; // Excitatory
                idx += 1;
            }
        }

        // Neuron 1 connection
        if idx < synapse_count {
            synapse_array.source_neurons[idx] = 1;
            synapse_array.target_neurons[idx] = 4;
            synapse_array.weights[idx] = 10;
            synapse_array.postsynaptic_potentials[idx] = 5;
            synapse_array.types[idx] = 0;
            idx += 1;
        }

        // Neuron 2 connection (inhibitory)
        if idx < synapse_count {
            synapse_array.source_neurons[idx] = 2;
            synapse_array.target_neurons[idx] = 5;
            synapse_array.weights[idx] = 10;
            synapse_array.postsynaptic_potentials[idx] = 5;
            synapse_array.types[idx] = 1; // Inhibitory
            idx += 1;
        }

        synapse_array.count = idx;

        (neuron_array, synapse_array)
    }

    #[test]
    #[ignore] // Requires actual CUDA hardware
    fn test_synaptic_propagation_correctness() {
        if !is_cuda_available() {
            println!("⚠️  CUDA not available, skipping test");
            return;
        }

        println!("🧪 Testing synaptic propagation correctness (CPU vs GPU)...");

        // Create test genome
        let (neuron_array, synapse_array) = create_test_genome(100, 200);

        // Setup CPU backend
        let mut cpu_backend = CPUBackend::new();
        <CPUBackend as ComputeBackend<f32, NeuronArray<f32>, SynapseArray>>::initialize_persistent_data(&mut cpu_backend, &neuron_array, &synapse_array)
            .expect("Failed to initialize CPU backend");

        // Setup CUDA backend
        let mut cuda_backend = CUDABackend::new(100, 200).expect("Failed to create CUDA backend");
        cuda_backend
            .initialize_persistent_data(&neuron_array, &synapse_array)
            .expect("Failed to initialize CUDA backend");

        // Test case: Single neuron firing (neuron 0)
        let fired_neurons = vec![0];

        // CPU execution
        let mut cpu_fcl = FireCandidateList::new();
        let cpu_synapse_count = <CPUBackend as ComputeBackend<f32>>::process_synaptic_propagation(
            &mut cpu_backend,
            &fired_neurons,
            &synapse_array,
            &mut cpu_fcl,
        )
        .expect("CPU synaptic propagation failed");

        // GPU execution
        let mut gpu_fcl = FireCandidateList::new();
        let gpu_synapse_count = cuda_backend
            .process_synaptic_propagation(&fired_neurons, &synapse_array, &mut gpu_fcl)
            .expect("GPU synaptic propagation failed");

        // Compare results
        println!(
            "CPU: {} synapses processed, {} FCL candidates",
            cpu_synapse_count,
            cpu_fcl.len()
        );
        println!(
            "GPU: {} synapses processed, {} FCL candidates",
            gpu_synapse_count,
            gpu_fcl.len()
        );

        // Verify FCL sizes match
        assert_eq!(
            cpu_fcl.len(),
            gpu_fcl.len(),
            "FCL sizes don't match! CPU: {}, GPU: {}",
            cpu_fcl.len(),
            gpu_fcl.len()
        );

        // Verify FCL contents match (within floating point tolerance)
        for (cpu_id, cpu_pot) in cpu_fcl.iter() {
            let gpu_pot = gpu_fcl
                .iter()
                .find(|(id, _)| id == &cpu_id)
                .map(|(_, pot)| pot)
                .expect(&format!("GPU FCL missing neuron {}", cpu_id.0));

            let diff = (cpu_pot - gpu_pot).abs();
            assert!(
                diff < 0.01,
                "Potential mismatch for neuron {}: CPU={}, GPU={}, diff={}",
                cpu_id.0,
                cpu_pot,
                gpu_pot,
                diff
            );
        }

        println!("✅ Synaptic propagation correctness verified!");
    }

    #[test]
    #[ignore] // Requires actual CUDA hardware
    fn test_neural_dynamics_correctness() {
        if !is_cuda_available() {
            println!("⚠️  CUDA not available, skipping test");
            return;
        }

        println!("🧪 Testing neural dynamics correctness (CPU vs GPU)...");

        // Create test genome
        let (mut neuron_array, synapse_array) = create_test_genome(100, 200);

        // Setup CPU backend
        let mut cpu_backend = CPUBackend::new();
        <CPUBackend as ComputeBackend<f32, NeuronArray<f32>, SynapseArray>>::initialize_persistent_data(&mut cpu_backend, &neuron_array, &synapse_array)
            .expect("Failed to initialize CPU backend");

        // Setup CUDA backend
        let mut cuda_backend = CUDABackend::new(100, 200).expect("Failed to create CUDA backend");
        cuda_backend
            .initialize_persistent_data(&neuron_array, &synapse_array)
            .expect("Failed to initialize CUDA backend");

        // Create identical FCL for both backends
        let mut cpu_fcl = FireCandidateList::new();
        cpu_fcl.add_candidate(NeuronId(1), 1.5); // Above threshold (1.0)
        cpu_fcl.add_candidate(NeuronId(2), 0.8); // Below threshold
        cpu_fcl.add_candidate(NeuronId(3), 1.2); // Above threshold

        let gpu_fcl = cpu_fcl.clone();

        // Clone neuron array for CPU
        let mut cpu_neuron_array = neuron_array.clone();

        // CPU execution
        let (cpu_fired, _cpu_fcl_size, _) =
            <CPUBackend as ComputeBackend<f32>>::process_neural_dynamics(
                &mut cpu_backend,
                &cpu_fcl,
                &mut cpu_neuron_array,
                1, // burst_count
            )
            .expect("CPU neural dynamics failed");

        // GPU execution
        let (gpu_fired, _gpu_fcl_size, _) = cuda_backend
            .process_neural_dynamics(
                &gpu_fcl,
                &mut neuron_array,
                1, // burst_count
            )
            .expect("GPU neural dynamics failed");

        // Compare results
        println!("CPU: {} neurons fired", cpu_fired.len());
        println!("GPU: {} neurons fired", gpu_fired.len());

        // Sort for comparison
        let mut cpu_sorted = cpu_fired.clone();
        let mut gpu_sorted = gpu_fired.clone();
        cpu_sorted.sort();
        gpu_sorted.sort();

        println!("CPU fired: {:?}", cpu_sorted);
        println!("GPU fired: {:?}", gpu_sorted);

        // Verify fired neuron counts match
        assert_eq!(
            cpu_fired.len(),
            gpu_fired.len(),
            "Fired neuron counts don't match! CPU: {}, GPU: {}",
            cpu_fired.len(),
            gpu_fired.len()
        );

        // Verify same neurons fired
        assert_eq!(
            cpu_sorted, gpu_sorted,
            "Different neurons fired! CPU: {:?}, GPU: {:?}",
            cpu_sorted, gpu_sorted
        );

        println!("✅ Neural dynamics correctness verified!");
    }

    #[test]
    #[ignore] // Requires actual CUDA hardware
    fn test_full_burst_cycle_correctness() {
        if !is_cuda_available() {
            println!("⚠️  CUDA not available, skipping test");
            return;
        }

        println!("🧪 Testing full burst cycle correctness (CPU vs GPU)...");

        // Create test genome
        let (mut neuron_array, synapse_array) = create_test_genome(100, 200);

        // Setup CPU backend
        let mut cpu_backend = CPUBackend::new();
        <CPUBackend as ComputeBackend<f32, NeuronArray<f32>, SynapseArray>>::initialize_persistent_data(&mut cpu_backend, &neuron_array, &synapse_array)
            .expect("Failed to initialize CPU backend");

        // Setup CUDA backend
        let mut cuda_backend = CUDABackend::new(100, 200).expect("Failed to create CUDA backend");
        cuda_backend
            .initialize_persistent_data(&neuron_array, &synapse_array)
            .expect("Failed to initialize CUDA backend");

        // Simulate 5 burst cycles
        let mut cpu_neuron_array = neuron_array.clone();
        let initial_fired = vec![0]; // Start with neuron 0 firing

        for burst in 1..=5 {
            println!("\n--- Burst {} ---", burst);

            // CPU burst cycle
            let mut cpu_fcl = FireCandidateList::new();
            <CPUBackend as ComputeBackend<f32, NeuronArray<f32>, SynapseArray>>::process_synaptic_propagation(
                &mut cpu_backend,
                &initial_fired,
                &synapse_array,
                &mut cpu_fcl
            ).expect("CPU synaptic propagation failed");

            let (cpu_fired, _, _) = <CPUBackend as ComputeBackend<f32>>::process_neural_dynamics(
                &mut cpu_backend,
                &cpu_fcl,
                &mut cpu_neuron_array,
                burst,
            )
            .expect("CPU neural dynamics failed");

            // GPU burst cycle
            let mut gpu_fcl = FireCandidateList::new();
            cuda_backend
                .process_synaptic_propagation(&initial_fired, &synapse_array, &mut gpu_fcl)
                .expect("GPU synaptic propagation failed");

            let (gpu_fired, _, _) = cuda_backend
                .process_neural_dynamics(&gpu_fcl, &mut neuron_array, burst)
                .expect("GPU neural dynamics failed");

            // Compare
            let mut cpu_sorted = cpu_fired.clone();
            let mut gpu_sorted = gpu_fired.clone();
            cpu_sorted.sort();
            gpu_sorted.sort();

            println!("CPU fired: {:?}", cpu_sorted);
            println!("GPU fired: {:?}", gpu_sorted);

            assert_eq!(
                cpu_sorted, gpu_sorted,
                "Burst {} diverged! CPU: {:?}, GPU: {:?}",
                burst, cpu_sorted, gpu_sorted
            );
        }

        println!("\n✅ Full burst cycle correctness verified for 5 bursts!");
    }

    #[test]
    #[ignore] // Requires actual CUDA hardware
    fn test_large_genome_correctness() {
        if !is_cuda_available() {
            println!("⚠️  CUDA not available, skipping test");
            return;
        }

        println!("🧪 Testing large genome correctness (10K neurons)...");

        let neuron_count = 10_000;
        let synapse_count = 50_000;

        let (mut neuron_array, synapse_array) = create_test_genome(neuron_count, synapse_count);

        // Setup CPU backend
        let mut cpu_backend = CPUBackend::new();
        <CPUBackend as ComputeBackend<f32, NeuronArray<f32>, SynapseArray>>::initialize_persistent_data(&mut cpu_backend, &neuron_array, &synapse_array)
            .expect("Failed to initialize CPU backend");

        // Setup CUDA backend
        let mut cuda_backend =
            CUDABackend::new(neuron_count, synapse_count).expect("Failed to create CUDA backend");
        cuda_backend
            .initialize_persistent_data(&neuron_array, &synapse_array)
            .expect("Failed to initialize CUDA backend");

        // Test with multiple firing neurons
        let fired_neurons: Vec<u32> = (0..100).collect(); // First 100 neurons fire

        // CPU execution
        let mut cpu_fcl = FireCandidateList::new();
        let cpu_start = std::time::Instant::now();
        <CPUBackend as ComputeBackend<f32, NeuronArray<f32>, SynapseArray>>::process_synaptic_propagation(
            &mut cpu_backend,
            &fired_neurons,
            &synapse_array,
            &mut cpu_fcl
        ).expect("CPU synaptic propagation failed");
        let cpu_duration = cpu_start.elapsed();

        // GPU execution
        let mut gpu_fcl = FireCandidateList::new();
        let gpu_start = std::time::Instant::now();
        cuda_backend
            .process_synaptic_propagation(&fired_neurons, &synapse_array, &mut gpu_fcl)
            .expect("GPU synaptic propagation failed");
        let gpu_duration = gpu_start.elapsed();

        println!("CPU time: {:?}", cpu_duration);
        println!("GPU time: {:?}", gpu_duration);
        println!(
            "Speedup: {:.2}x",
            cpu_duration.as_secs_f64() / gpu_duration.as_secs_f64()
        );

        // Verify correctness
        assert_eq!(
            cpu_fcl.len(),
            gpu_fcl.len(),
            "Large genome FCL sizes don't match!"
        );

        println!("✅ Large genome correctness verified!");
    }
}

#[cfg(all(test, not(feature = "cuda")))]
#[test]
fn test_cuda_not_compiled() {
    println!("⚠️  CUDA tests skipped - compile with --features cuda");
}
