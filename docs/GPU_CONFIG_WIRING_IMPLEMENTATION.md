# GPU Config Wiring - Implementation Plan

**Task**: Wire GPU configuration from TOML → NPU → Backend Selection  
**Estimated Time**: 1-2 weeks  
**Complexity**: Low (straightforward integration)

---

## Step-by-Step Implementation

### Step 1: Create `GpuConfig` in `feagi-burst-engine`

**File**: `feagi-core/crates/feagi-burst-engine/src/backend/mod.rs`

**Add after line 247** (after `BackendConfig` definition):

```rust
/// GPU configuration from TOML (simplified interface for burst engine)
#[derive(Debug, Clone)]
pub struct GpuConfig {
    /// Enable GPU processing
    pub use_gpu: bool,
    
    /// Enable hybrid CPU/GPU auto-selection
    pub hybrid_enabled: bool,
    
    /// Threshold in synapses to consider GPU (hybrid mode)
    pub gpu_threshold: usize,
    
    /// Fraction of GPU memory to use (0.0-1.0)
    pub gpu_memory_fraction: f64,
}

impl Default for GpuConfig {
    fn default() -> Self {
        Self {
            use_gpu: true,
            hybrid_enabled: true,
            gpu_threshold: 1_000_000,
            gpu_memory_fraction: 0.8,
        }
    }
}

impl GpuConfig {
    /// Convert to BackendConfig and BackendType for backend selection
    pub fn to_backend_config(&self) -> (BackendType, BackendConfig) {
        let backend_type = if !self.use_gpu {
            // GPU explicitly disabled
            BackendType::CPU
        } else if self.hybrid_enabled {
            // Hybrid mode: auto-select based on genome size
            BackendType::Auto
        } else {
            // GPU always on (if available)
            #[cfg(feature = "gpu")]
            {
                BackendType::WGPU
            }
            #[cfg(not(feature = "gpu"))]
            {
                tracing::warn!("GPU requested but 'gpu' feature not enabled, falling back to CPU");
                BackendType::CPU
            }
        };
        
        let backend_config = BackendConfig {
            gpu_neuron_threshold: self.gpu_threshold / 100, // Rough estimate: 100 synapses/neuron
            gpu_synapse_threshold: self.gpu_threshold,
            force_cpu: !self.use_gpu,
            force_gpu: self.use_gpu && !self.hybrid_enabled,
            ..Default::default()
        };
        
        (backend_type, backend_config)
    }
}
```

**Testing**:
```rust
#[cfg(test)]
mod gpu_config_tests {
    use super::*;
    
    #[test]
    fn test_gpu_disabled() {
        let config = GpuConfig {
            use_gpu: false,
            ..Default::default()
        };
        let (backend_type, backend_config) = config.to_backend_config();
        assert_eq!(backend_type, BackendType::CPU);
        assert!(backend_config.force_cpu);
    }
    
    #[test]
    fn test_gpu_hybrid_mode() {
        let config = GpuConfig {
            use_gpu: true,
            hybrid_enabled: true,
            gpu_threshold: 500_000,
            ..Default::default()
        };
        let (backend_type, backend_config) = config.to_backend_config();
        assert_eq!(backend_type, BackendType::Auto);
        assert_eq!(backend_config.gpu_synapse_threshold, 500_000);
    }
    
    #[test]
    #[cfg(feature = "gpu")]
    fn test_gpu_always_on() {
        let config = GpuConfig {
            use_gpu: true,
            hybrid_enabled: false,
            ..Default::default()
        };
        let (backend_type, _) = config.to_backend_config();
        assert_eq!(backend_type, BackendType::WGPU);
    }
}
```

---

### Step 2: Update `RustNPU::new()` to Accept GPU Config

**File**: `feagi-core/crates/feagi-burst-engine/src/npu.rs`

**Current signature** (around line 100):
```rust
pub fn new(
    neuron_capacity: usize,
    synapse_capacity: usize,
    cortical_area_count: usize,
) -> Self
```

**Updated signature**:
```rust
pub fn new(
    neuron_capacity: usize,
    synapse_capacity: usize,
    cortical_area_count: usize,
    gpu_config: Option<&backend::GpuConfig>,  // NEW PARAMETER
) -> Self {
    use tracing::info;
    
    // Determine backend based on GPU config
    let (backend_type, backend_config) = if let Some(config) = gpu_config {
        config.to_backend_config()
    } else {
        // No config provided: default to CPU
        (backend::BackendType::CPU, backend::BackendConfig::default())
    };
    
    info!("🎮 Creating NPU with backend: {}", backend_type);
    if let Some(config) = gpu_config {
        info!("   GPU enabled: {}", config.use_gpu);
        info!("   Hybrid mode: {}", config.hybrid_enabled);
        info!("   GPU threshold: {} synapses", config.gpu_threshold);
    }
    
    // Create backend
    let backend = backend::create_backend(
        backend_type,
        neuron_capacity,
        synapse_capacity,
        &backend_config,
    ).expect("Failed to create compute backend");
    
    info!("   ✓ Backend selected: {}", backend.backend_name());
    
    // Rest of NPU initialization (unchanged)
    Self {
        neuron_array: NeuronArray::new(neuron_capacity),
        synapse_array: SynapseArray::new(synapse_capacity),
        fire_candidate_list: FireCandidateList::new(neuron_capacity),
        fire_queue: FireQueue::new(neuron_capacity),
        fire_ledger: FireLedger::new(neuron_capacity),
        cortical_areas: vec![CorticalAreaId(0); cortical_area_count],
        burst_count: 0,
        backend,  // Use the selected backend
    }
}
```

**Also update `import_connectome()`** (around line 250):
```rust
pub fn import_connectome(connectome: Connectome) -> Self {
    Self::import_connectome_with_config(connectome, None)
}

pub fn import_connectome_with_config(
    connectome: Connectome,
    gpu_config: Option<&backend::GpuConfig>,
) -> Self {
    // Create NPU with GPU config
    let mut npu = Self::new(
        connectome.neurons.capacity,
        connectome.synapses.capacity,
        connectome.cortical_area_names.len(),
        gpu_config,  // Pass GPU config
    );
    
    // Import data
    npu.neuron_array = connectome.neurons;
    npu.synapse_array = connectome.synapses;
    // ... rest unchanged
    
    npu
}
```

---

### Step 3: Wire Config in `feagi/src/main.rs`

**File**: `/Users/nadji/code/FEAGI-2.0/feagi/src/main.rs`

**Add import at top** (after line 26):
```rust
use feagi_burst_engine::backend::GpuConfig;
```

**Update NPU initialization** (replace lines 152-160):
```rust
// Initialize NPU with GPU configuration
info!("  Initializing NPU...");

// Create GPU config from TOML settings
let gpu_config = GpuConfig {
    use_gpu: config.resources.use_gpu,
    hybrid_enabled: config.neural.hybrid.enabled,
    gpu_threshold: config.neural.hybrid.gpu_threshold,
    gpu_memory_fraction: config.resources.gpu_memory_fraction,
};

info!("  GPU Configuration:");
info!("    - GPU enabled: {}", gpu_config.use_gpu);
info!("    - Hybrid mode: {}", gpu_config.hybrid_enabled);
info!("    - GPU threshold: {} synapses", gpu_config.gpu_threshold);
info!("    - GPU memory fraction: {:.1}%", gpu_config.gpu_memory_fraction * 100.0);

let npu = Arc::new(Mutex::new(RustNPU::new(
    config.connectome.neuron_space,
    config.connectome.synapse_space,
    10, // cortical_area_count - will be resized as needed
    Some(&gpu_config),  // Pass GPU config
)));

info!("    ✓ NPU initialized (capacity: {} neurons, {} synapses)",
      config.connectome.neuron_space,
      config.connectome.synapse_space);
```

**Update genome loading** (replace lines 226-256 in `load_genome` function):
```rust
async fn load_genome(
    manager: &Arc<RwLock<ConnectomeManager>>,
    genome_path: &PathBuf,
) -> Result<()> {
    use feagi_evo::{load_genome_from_file, validate_genome};
    
    // Load genome from file
    let genome = load_genome_from_file(genome_path)
        .context("Failed to load genome file")?;
    
    // Validate genome
    let validation = validate_genome(&genome);
    if !validation.errors.is_empty() {
        error!("Genome validation errors:");
        for error in &validation.errors {
            error!("  - {}", error);
        }
        return Err(anyhow::anyhow!("Genome validation failed"));
    }
    
    if !validation.warnings.is_empty() {
        warn!("Genome validation warnings:");
        for warning in &validation.warnings {
            warn!("  - {}", warning);
        }
    }
    
    // Load genome into connectome (includes neuroembryogenesis)
    // Note: GPU backend will be auto-selected based on resulting genome size
    manager.write().load_from_genome(genome)
        .context("Failed to load genome into connectome")?;
    
    // Log backend selection result after genome is loaded
    info!("  Backend selection completed based on genome size");
    
    Ok(())
}
```

---

### Step 4: Update `feagi-inference-engine` Similarly

**File**: `/Users/nadji/code/FEAGI-2.0/feagi-inference-engine/src/main.rs`

**Add after line 6**:
```rust
use feagi_burst_engine::backend::GpuConfig;
```

**Add CLI argument** (after line 64):
```rust
/// Enable GPU acceleration (default: auto-detect based on brain size)
#[arg(long, default_value_t = true)]
gpu_enabled: bool,

/// GPU threshold in synapses (default: 1000000)
#[arg(long, default_value_t = 1000000)]
gpu_threshold: usize,
```

**Update NPU creation** (replace line 124):
```rust
// Create NPU from connectome with GPU config
info!("Initializing NPU with GPU configuration...");

let gpu_config = GpuConfig {
    use_gpu: args.gpu_enabled,
    hybrid_enabled: true,
    gpu_threshold: args.gpu_threshold,
    gpu_memory_fraction: 0.8,
};

info!("  GPU enabled: {}", gpu_config.use_gpu);
info!("  GPU threshold: {} synapses", gpu_config.gpu_threshold);

let mut npu = feagi_burst_engine::RustNPU::import_connectome_with_config(
    connectome,
    Some(&gpu_config),
);

info!("✓ NPU initialized successfully!");
```

---

## Step 5: Testing the Integration

### Test 1: GPU Disabled

**Config** (`feagi_configuration.toml`):
```toml
[resources]
use_gpu = false
```

**Expected Output**:
```
🎮 Creating NPU with backend: CPU
   GPU enabled: false
   ✓ Backend selected: CPU (SIMD)
```

**Command**:
```bash
./target/release/feagi --config feagi_configuration.toml --genome test_genome.json
```

---

### Test 2: GPU Hybrid Mode (Small Genome)

**Config**:
```toml
[neural.hybrid]
enabled = true
gpu_threshold = 1000000

[resources]
use_gpu = true
```

**Genome**: 100K neurons, 10M synapses (below threshold)

**Expected Output**:
```
🎮 Creating NPU with backend: Auto
   GPU enabled: true
   Hybrid mode: true
   GPU threshold: 1000000 synapses
🎯 Backend auto-selection: CPU (Small genome: 100000 neurons, 10000000 synapses)
   ✓ Backend selected: CPU (SIMD)
```

---

### Test 3: GPU Hybrid Mode (Large Genome)

**Genome**: 2M neurons, 200M synapses (above threshold)

**Expected Output**:
```
🎮 Creating NPU with backend: Auto
   GPU enabled: true
   Hybrid mode: true
   GPU threshold: 1000000 synapses
🎯 Backend auto-selection: WGPU (Large genome: 2000000 neurons, 200000000 synapses)
   Estimated speedup: 8.5x
🎮 Using WGPU backend (GPU accelerated)
   ✓ Backend selected: WGPU (Apple M4 Pro - Metal)
```

---

### Test 4: GPU Always On

**Config**:
```toml
[neural.hybrid]
enabled = false  # Disable auto-selection

[resources]
use_gpu = true
```

**Expected Output**:
```
🎮 Creating NPU with backend: WGPU
   GPU enabled: true
   Hybrid mode: false
🎮 Using WGPU backend (GPU accelerated)
   ✓ Backend selected: WGPU (Apple M4 Pro - Metal)
```

---

### Test 5: GPU Feature Not Compiled

**Build without GPU**:
```bash
cargo build --release  # Without --features gpu
```

**Config**:
```toml
[resources]
use_gpu = true
```

**Expected Output**:
```
🎮 Creating NPU with backend: CPU
   GPU enabled: true
⚠️  GPU requested but 'gpu' feature not enabled, falling back to CPU
   ✓ Backend selected: CPU (SIMD)
```

---

## Step 6: Update Cargo.toml Feature Flags

**File**: `/Users/nadji/code/FEAGI-2.0/feagi/Cargo.toml`

**Add after line 33** (in dependencies section):
```toml
# GPU acceleration (optional, enabled by default)
feagi-burst-engine = { path = "../feagi-core/crates/feagi-burst-engine", features = ["gpu"] }
```

**Add features section** (after line 53):
```toml
[features]
default = ["gpu"]
gpu = ["feagi-burst-engine/gpu"]
```

**Build commands**:
```bash
# With GPU (default)
cargo build --release

# Without GPU (CPU only)
cargo build --release --no-default-features

# Explicit GPU
cargo build --release --features gpu
```

---

## Step 7: Verification

### Check 1: Config Parsing
```bash
# Verify config is read correctly
./target/release/feagi --config feagi_configuration.toml 2>&1 | grep -A5 "GPU Configuration"
```

**Expected**:
```
GPU Configuration:
  - GPU enabled: true
  - Hybrid mode: true
  - GPU threshold: 1000000 synapses
  - GPU memory fraction: 80.0%
```

### Check 2: Backend Selection
```bash
# Verify backend is selected
./target/release/feagi --config feagi_configuration.toml 2>&1 | grep "Backend selected"
```

**Expected** (small genome):
```
✓ Backend selected: CPU (SIMD)
```

**Expected** (large genome):
```
✓ Backend selected: WGPU (Apple M4 Pro - Metal)
```

### Check 3: GPU Detection
```bash
# Check if GPU is available
./target/release/feagi --config feagi_configuration.toml 2>&1 | grep -i "gpu\|metal\|vulkan\|directx"
```

---

## Step 8: Error Handling

### If GPU Not Available

**Add to `feagi-burst-engine/src/backend/mod.rs`** (in `create_backend`):

```rust
pub fn create_backend(
    backend_type: BackendType,
    neuron_capacity: usize,
    synapse_capacity: usize,
    config: &BackendConfig,
) -> Result<Box<dyn ComputeBackend>> {
    let actual_type = if backend_type == BackendType::Auto {
        let decision = select_backend(neuron_capacity, synapse_capacity, config);
        info!(
            "🎯 Backend auto-selection: {} ({})",
            decision.backend_type, decision.reason
        );
        if decision.estimated_speedup > 1.0 {
            info!("   Estimated speedup: {:.1}x", decision.estimated_speedup);
        }
        decision.backend_type
    } else {
        backend_type
    };

    match actual_type {
        BackendType::CPU => {
            info!("🖥️  Using CPU backend (SIMD optimized)");
            Ok(Box::new(CPUBackend::new()))
        }
        #[cfg(feature = "gpu")]
        BackendType::WGPU => {
            info!("🎮 Using WGPU backend (GPU accelerated)");
            match WGPUBackend::new(neuron_capacity, synapse_capacity) {
                Ok(backend) => Ok(Box::new(backend)),
                Err(e) => {
                    warn!("⚠️  Failed to create GPU backend: {}", e);
                    warn!("   Falling back to CPU backend");
                    Ok(Box::new(CPUBackend::new()))
                }
            }
        }
        BackendType::Auto => {
            // Should not reach here, but fallback to CPU
            Ok(Box::new(CPUBackend::new()))
        }
    }
}
```

---

## Step 9: Documentation Updates

### Update `README.md` in `feagi/`:

**Add GPU section**:
```markdown
## GPU Acceleration

FEAGI supports automatic GPU acceleration via WGPU (Metal/Vulkan/DirectX 12).

### Configuration

Edit `feagi_configuration.toml`:

```toml
[neural.hybrid]
enabled = true                # Enable GPU auto-selection
gpu_threshold = 1000000       # Use GPU for genomes ≥1M synapses

[resources]
use_gpu = true                # Enable GPU globally
gpu_memory_fraction = 0.8     # Use 80% of GPU memory
```

### Building with GPU Support

```bash
# Default (GPU enabled)
cargo build --release

# CPU only (smaller binary)
cargo build --release --no-default-features
```

### Verification

Check logs on startup:
```
🎮 Creating NPU with backend: Auto
   GPU enabled: true
   ✓ Backend selected: WGPU (Apple M4 Pro - Metal)
```

### Troubleshooting

**GPU not detected**: Check drivers (Metal on macOS, Vulkan on Linux, DX12 on Windows)

**Performance worse than CPU**: Genome too small, increase threshold or disable GPU
```

---

## Implementation Checklist

- [ ] Step 1: Create `GpuConfig` struct
- [ ] Step 2: Update `RustNPU::new()` signature
- [ ] Step 3: Wire config in `feagi/src/main.rs`
- [ ] Step 4: Update `feagi-inference-engine`
- [ ] Step 5: Test all scenarios
- [ ] Step 6: Update Cargo.toml features
- [ ] Step 7: Run verification checks
- [ ] Step 8: Add error handling
- [ ] Step 9: Update documentation

**Estimated Time**: 1-2 weeks (5-10 days)

**Team Size**: 1 engineer

**Complexity**: Low (straightforward integration)

---

## Commit Messages

```
commit 1: Add GpuConfig struct to burst engine backend
commit 2: Update RustNPU to accept GPU configuration
commit 3: Wire GPU config from TOML to NPU in main binary
commit 4: Update feagi-inference-engine with GPU support
commit 5: Add comprehensive GPU config tests
commit 6: Update Cargo.toml with GPU feature flags
commit 7: Add error handling for GPU initialization failures
commit 8: Update documentation with GPU configuration guide
```

---

**Next Steps**: See verification script in `GPU_VERIFICATION_SCRIPT.md`


