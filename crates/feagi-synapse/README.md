# feagi-synapse

Platform-agnostic synaptic computation algorithms for FEAGI.

## Features

- ✅ `no_std` compatible (ESP32, RTOS, WASM)
- ✅ Zero allocations (stack-only)
- ✅ SIMD-friendly vectorization
- ✅ Pure functions (deterministic)

## Modules

- `contribution` - Synaptic current calculation (weight × conductance × sign)
- `weight` - Weight conversion, normalization, plasticity updates

## Example

```rust
use feagi_synapse::{compute_synaptic_contribution, SynapseType};

// Calculate excitatory contribution
let contribution = compute_synaptic_contribution(255, 255, SynapseType::Excitatory);
assert_eq!(contribution, 65025.0); // Maximum excitatory (255 × 255, NO normalization)

// Calculate inhibitory contribution
let contribution = compute_synaptic_contribution(255, 255, SynapseType::Inhibitory);
assert_eq!(contribution, -65025.0); // Maximum inhibitory

// IMPORTANT: Direct cast (NO division by 255), matches Python behavior
```

## Platform Support

| Platform | Status | Notes |
|----------|--------|-------|
| Desktop | ✅ | Full support |
| ESP32 | ✅ | `no_std` mode |
| RTOS | ✅ | FreeRTOS, Zephyr |
| WASM | ✅ | Browser, Node.js |
| HPC | ✅ | MPI clusters |

## Performance

- Single synapse: ~30 cycles (125 ns @ 240 MHz)
- Batch processing: SIMD-optimized
- Memory: <1 KB stack, 0 heap

## License

Apache License 2.0

