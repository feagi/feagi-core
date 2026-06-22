# CUDA Backend Testing & Validation Guide

## Overview

This guide provides step-by-step procedures for testing the FEAGI CUDA backend on **any** NVIDIA GPU with CUDA support.

**Works on:** Tesla P100/V100, A100/A40/A6000, H100/H200, RTX 3090/4090, and any future NVIDIA GPU

---

## Prerequisites

### Software Requirements

```bash
# Check CUDA installation
nvcc --version
# Expected: CUDA 11.8 or later

# Check GPU driver
nvidia-smi
# Expected: Display your GPU(s)

# Check Rust
rustc --version
# Expected: 1.75 or later
```

### Verify GPU Compute Capability

```bash
# Check compute capability
nvidia-smi --query-gpu=compute_cap --format=csv

# Minimum required: 7.0 (Volta/2017+)
# If < 7.0, FEAGI will refuse to run with helpful error message
```

---

## Phase 1: Build Validation (No GPU Execution)

### Step 1.1: Clean Build

```bash
cd /path/to/FEAGI-2.0/feagi-core/crates/feagi-burst-engine

# Clean previous builds
cargo clean

# Build with CUDA feature
cargo build --release --features cuda 2>&1 | tee build.log

# Expected output:
# - "Compiling synaptic_propagation_fcl.cu to PTX..."
# - "✅ Compiled successfully"
# - "Compiling neural_dynamics_fcl.cu to PTX..."
# - "✅ Compiled successfully"
# - Finished release [optimized] target(s)
```

**Success criteria:**
- ✅ No compilation errors
- ✅ PTX files generated in `target/release/build/.../out/`
- ✅ Warnings are OK (unused imports, etc.)

**Common issues:**
- `nvcc not found`: Install CUDA Toolkit
- `PTX compilation failed`: Check .cu syntax (shouldn't happen with provided files)

### Step 1.2: Structure Tests (No GPU)

```bash
# Run tests that don't require GPU
cargo test --release --features cuda

# Expected: 3 tests pass, 3 fail (no CUDA runtime), 3 ignored
```

**Success criteria:**
- ✅ `test_cuda_feature_enabled` - PASS
- ✅ `test_backend_trait_object_safety` - PASS  
- ✅ `test_cuda_compile_time_validation` - PASS
- ⚠️  `test_cuda_availability_check` - FAIL (expected, no GPU on build machine)
- ⚠️  `test_enumerate_cuda_devices` - FAIL (expected)
- ⚠️  `test_cuda_backend_size_limits` - FAIL (expected)

---

## Phase 2: GPU Detection

### Step 2.1: Enumerate GPUs

```bash
cargo test --release --features cuda test_enumerate_cuda_devices -- --nocapture

# Expected output (example for 4× A100):
# Found 4 CUDA device(s)
#   GPU 0: NVIDIA A100-SXM4-40GB (40 GB)
#   GPU 1: NVIDIA A100-SXM4-40GB (40 GB)
#   GPU 2: NVIDIA A100-SXM4-40GB (40 GB)
#   GPU 3: NVIDIA A100-SXM4-40GB (40 GB)
```

**Success criteria:**
- ✅ Lists all GPUs in system
- ✅ Shows correct GPU names
- ✅ Memory amounts are reasonable (not 0 or garbage)

**Troubleshooting:**
- No devices found → Check `nvidia-smi`, verify CUDA driver
- Wrong count → Check `CUDA_VISIBLE_DEVICES` env var

### Step 2.2: Check Availability

```bash
cargo test --release --features cuda test_cuda_availability_check -- --nocapture

# Expected:
# CUDA available: true
```

**Success criteria:**
- ✅ Returns `true`
- ✅ No panics or errors

---

## Phase 3: Backend Creation

### Step 3.1: Create Single GPU Backend

```bash
cargo test --release --features cuda test_cuda_backend_creation -- --ignored --nocapture

# Expected output:
# 🔧 Initializing CUDA backend on GPU 0...
# ✅ Created CUDA backend on GPU 0
#    GPU: NVIDIA A100-SXM4-40GB (GPU 0)
#    Compute Capability: 8.0
#    Total Memory: 40.0 GB
#    Capacity: 10000 neurons, 100000 synapses
# ✅ Created CUDA backend: NVIDIA A100-SXM4-40GB (GPU 0)
```

**Success criteria:**
- ✅ Backend created successfully
- ✅ GPU detected with correct name
- ✅ Compute capability ≥ 7.0
- ✅ Memory amount correct

### Step 3.2: Test Multi-GPU Detection

```bash
cargo test --release --features cuda test_cuda_multi_device -- --ignored --nocapture

# Expected (for 4-GPU system):
# Testing multi-GPU with 4 devices
# ✅ Created backend on GPU 0: NVIDIA A100-SXM4-40GB (GPU 0)
# ✅ Created backend on GPU 1: NVIDIA A100-SXM4-40GB (GPU 1)
# ✅ Created backend on GPU 2: NVIDIA A100-SXM4-40GB (GPU 2)
# ✅ Created backend on GPU 3: NVIDIA A100-SXM4-40GB (GPU 3)
```

**Success criteria:**
- ✅ Can create backend on each GPU
- ✅ Each backend shows correct GPU ID
- ✅ No interference between GPUs

---

## Phase 4: Memory Management

### Step 4.1: Data Upload Test

```bash
cargo test --release --features cuda test_cuda_backend_initialization -- --ignored --nocapture

# Expected:
# 🔧 Initializing CUDA backend on GPU 0...
# ✅ Created CUDA backend...
# 📦 Loading CUDA kernels from PTX...
# ✅ CUDA kernels loaded successfully
# 📤 Uploading 1000 neurons to GPU memory...
# ✅ Uploaded 1000 neurons (0 MB)
# 📤 Uploading 10000 synapses to GPU memory...
# ✅ Uploaded 10000 synapses (0 MB)
# ✅ Successfully initialized CUDA backend
```

**Success criteria:**
- ✅ Kernels load successfully
- ✅ Neuron data uploads
- ✅ Synapse data uploads
- ✅ No CUDA errors

**Troubleshooting:**
- `Failed to load PTX` → Check PTX compilation in build
- `Out of memory` → Reduce test size or use smaller genome
- `Invalid argument` → Check buffer alignment (report as bug)

### Step 4.2: Size Limits Test

```bash
cargo test --release --features cuda test_cuda_backend_size_limits -- --ignored --nocapture

# Expected:
# Testing: 100K neurons, 10M synapses
#   ✅ 100K neurons, 10M synapses fits in CUDA backend
# Testing: 500K neurons, 50M synapses
#   ✅ 500K neurons, 50M synapses fits in CUDA backend
# Testing: 1M neurons, 100M synapses
#   ⚠️  1M neurons, 100M synapses exceeds limits: ...
```

**Success criteria:**
- ✅ Small/medium genomes accepted
- ✅ Very large genomes rejected gracefully (not crash)
- ✅ Error messages are helpful

---

## Phase 5: Kernel Execution (CRITICAL)

### Step 5.1: Minimal Execution Test

Create test file `test_minimal_execution.rs`:

```rust
use feagi_burst_engine::backend::{CUDABackend, ComputeBackend};
use feagi_types::{NeuronArray, SynapseArray, FireCandidateList};

#[test]
#[ignore]
fn test_minimal_kernel_execution() {
    // Create minimal genome
    let mut neurons = NeuronArray::new(100);
    let mut synapses = SynapseArray::new(1000);
    
    // Initialize neurons
    for i in 0..100 {
        neurons.membrane_potentials[i] = 0.0;
        neurons.thresholds[i] = 10.0;
        neurons.leak_coefficients[i] = 0.1;
        neurons.resting_potentials[i] = 0.0;
        neurons.excitabilities[i] = 1.0;
        neurons.valid_mask[i] = true;
    }
    neurons.count = 100;
    
    // Initialize synapses (simple chain: 0→1→2→...→99)
    for i in 0..99 {
        synapses.source_neurons[i] = i as u32;
        synapses.target_neurons[i] = (i + 1) as u32;
        synapses.weights[i] = 128;
        synapses.postsynaptic_potentials[i] = 200;
        synapses.types[i] = 0;  // Excitatory
        synapses.valid_mask[i] = true;
        synapses.source_index.entry(i as u32).or_insert_with(Vec::new).push(i);
    }
    synapses.count = 99;
    
    // Create CUDA backend
    let mut backend = CUDABackend::new(100, 1000).unwrap();
    backend.initialize_persistent_data(&neurons, &synapses).unwrap();
    
    // Test synaptic propagation
    let mut fcl = FireCandidateList::new();
    let fired = vec![0u32];  // Fire neuron 0
    
    let processed = backend.process_synaptic_propagation(&fired, &synapses, &mut fcl);
    
    assert!(processed.is_ok(), "Synaptic propagation failed: {:?}", processed.err());
    assert!(!fcl.is_empty(), "FCL should not be empty after propagation");
    
    println!("✅ Synaptic propagation produced {} candidates", fcl.len());
    
    // Test neural dynamics
    let result = backend.process_neural_dynamics(&fcl, &mut neurons, 1);
    
    assert!(result.is_ok(), "Neural dynamics failed: {:?}", result.err());
    
    let (fired_out, _, _) = result.unwrap();
    println!("✅ Neural dynamics produced {} fired neurons", fired_out.len());
    
    assert!(!fired_out.is_empty(), "Should have at least one fired neuron");
}
```

Run:
```bash
cargo test --release --features cuda test_minimal_kernel_execution -- --ignored --nocapture
```

**Success criteria:**
- ✅ Synaptic kernel launches without error
- ✅ FCL is populated (>0 candidates)
- ✅ Neural kernel launches without error
- ✅ Fired neurons are returned

**This is the CRITICAL test! If this passes, the CUDA backend works!**

---

## Phase 6: Correctness Validation

### Step 6.1: CPU vs CUDA Comparison

```rust
#[test]
#[ignore]
fn test_cuda_matches_cpu() {
    // Create test genome
    let neurons = create_test_neurons(1000);
    let synapses = create_test_synapses(10000);
    
    // Run on CPU
    let mut cpu_backend = CPUBackend::new();
    cpu_backend.initialize_persistent_data(&neurons, &synapses).unwrap();
    
    let mut fcl_cpu = FireCandidateList::new();
    let fired = vec![0, 10, 20, 30];
    cpu_backend.process_synaptic_propagation(&fired, &synapses, &mut fcl_cpu).unwrap();
    let (fired_cpu, _, _) = cpu_backend.process_neural_dynamics(&fcl_cpu, &mut neurons.clone(), 1).unwrap();
    
    // Run on CUDA
    let mut cuda_backend = CUDABackend::new(1000, 10000).unwrap();
    cuda_backend.initialize_persistent_data(&neurons, &synapses).unwrap();
    
    let mut fcl_cuda = FireCandidateList::new();
    cuda_backend.process_synaptic_propagation(&fired, &synapses, &mut fcl_cuda).unwrap();
    let (fired_cuda, _, _) = cuda_backend.process_neural_dynamics(&fcl_cuda, &mut neurons.clone(), 1).unwrap();
    
    // Compare results
    assert_eq!(fired_cpu.len(), fired_cuda.len(), "Different number of fired neurons");
    
    for &neuron_id in &fired_cpu {
        assert!(fired_cuda.contains(&neuron_id), "CPU fired {}, CUDA didn't", neuron_id);
    }
    
    println!("✅ CUDA results match CPU ({} fired neurons)", fired_cpu.len());
}
```

**Success criteria:**
- ✅ Same neurons fire on CPU and CUDA
- ✅ FCL sizes match (within tolerance)
- ✅ Consistent across multiple runs

---

## Phase 7: Performance Benchmarking

### Step 7.1: Run Benchmarks

```bash
cargo bench --release --features cuda -- cuda_backend

# Expected output (example on A100):
# cuda_backend/10K_neurons    time: [0.8ms 0.9ms 1.0ms]
# cuda_backend/50K_neurons    time: [2.5ms 2.8ms 3.1ms]
# cuda_backend/100K_neurons   time: [4.5ms 5.0ms 5.5ms]
```

### Step 7.2: Compare CPU vs CUDA

```bash
cargo bench --release --features cuda -- "cpu_vs_cuda"

# Expected (on A100, 100K neurons):
# CPU:  time: [8.5ms 9.0ms 9.5ms]
# CUDA: time: [4.5ms 5.0ms 5.5ms]
# Speedup: 1.8x
```

**Success criteria:**
- ✅ CUDA is faster than CPU for large genomes (>50K neurons)
- ✅ Times are consistent across runs
- ✅ No timeouts or hangs

---

## Phase 8: Multi-GPU Testing (Advanced)

### Step 8.1: Manual Multi-GPU

```bash
# Terminal 1: GPU 0
CUDA_VISIBLE_DEVICES=0 cargo run --release --features cuda

# Terminal 2: GPU 1
CUDA_VISIBLE_DEVICES=1 cargo run --release --features cuda

# Both should run independently without conflict
```

### Step 8.2: P2P Bandwidth Test

```bash
# Use CUDA samples
cd /usr/local/cuda/samples/1_Utilities/p2pBandwidthLatencyTest
make
./p2pBandwidthLatencyTest

# Check for NVLink connectivity
nvidia-smi topo -m
```

---

## Troubleshooting

### Issue: "PTX compilation failed"

**Solution:**
```bash
# Check nvcc
which nvcc
nvcc --version

# Manually compile to see error
cd src/backend/shaders/cuda
nvcc --ptx synaptic_propagation_fcl.cu -o test.ptx
```

### Issue: "Kernel launch failed"

**Possible causes:**
1. Invalid parameters → Check buffer sizes match
2. Out of memory → Reduce genome size
3. Compute capability mismatch → Check GPU requirements

**Debug:**
```bash
# Enable CUDA error checking
export CUDA_LAUNCH_BLOCKING=1
cargo test --features cuda ... -- --nocapture
```

### Issue: "Different results than CPU"

**Possible causes:**
1. Floating-point precision differences (expected, minor)
2. Race conditions in atomic operations (bug!)
3. Incorrect kernel logic (bug!)

**Debug:**
- Run with small deterministic genome
- Check FCL values match
- Verify fired neuron IDs match exactly

---

## Success Checklist

Before declaring CUDA backend production-ready:

- [ ] Builds cleanly on GPU-enabled system
- [ ] Enumerates all GPUs correctly
- [ ] Creates backend on each GPU
- [ ] Uploads data without errors
- [ ] Kernels launch successfully
- [ ] FCL is populated correctly
- [ ] Fired neurons match CPU results
- [ ] Performance is better than CPU at scale
- [ ] No memory leaks (run valgrind or cuda-memcheck)
- [ ] Works on multiple GPU types (Tesla, A-series, H-series, RTX)
- [ ] Multi-GPU works independently

---

## Performance Targets

| GPU Type | 100K neurons @ 1% | 500K neurons @ 1% | 1M neurons @ 1% |
|----------|-------------------|-------------------|-----------------|
| Tesla V100 | < 5ms | < 20ms | < 40ms |
| A100 | < 3ms | < 12ms | < 25ms |
| H100 | < 2ms | < 8ms | < 15ms |
| RTX 4090 | < 3ms | < 15ms | < 30ms |

If slower than these targets, investigate:
- Kernel launch overhead
- Memory transfer bottlenecks
- Sub-optimal block sizes

---

## Reporting Issues

When reporting CUDA-related issues, include:

```bash
# System info
nvidia-smi
nvcc --version
rustc --version

# Build log
cargo build --features cuda 2>&1 | tee build.log

# Test output
cargo test --features cuda ... -- --nocapture 2>&1 | tee test.log

# Error messages (full stack trace)
RUST_BACKTRACE=full cargo test ...
```

---

**Status:** Testing guide complete
**Next:** Run through these tests on actual CUDA hardware
**Timeline:** 1-2 days per phase with hardware access

