# feagi-neural

Platform-agnostic neural dynamics algorithms for FEAGI.

## Features

- ✅ `no_std` compatible (ESP32, RTOS, WASM)
- ✅ Zero allocations (stack-only)
- ✅ SIMD-friendly batch operations
- ✅ Pure functions (no side effects)

## Modules

- `dynamics` - LIF neuron updates, leak, threshold checks
- `firing` - Refractory periods, consecutive fire limits
- `utils` - PCG random number generation, excitability

## Example

```rust
use feagi_neural::update_neuron_lif;

let mut potential = 0.5;
let threshold = 1.0;
let leak = 0.1;
let input = 0.6;

let fired = update_neuron_lif(&mut potential, threshold, leak, 0.0, input);
assert!(fired); // 0.5 + 0.6 = 1.1 > 1.0
assert_eq!(potential, 0.0); // Reset after firing
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

- Single neuron update: ~50 cycles (208 ns @ 240 MHz)
- Batch processing: SIMD-optimized
- Memory: <1 KB stack, 0 heap

## License

Apache License 2.0

