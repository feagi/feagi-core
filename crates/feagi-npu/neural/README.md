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
let mut fraction = 0.0;
let threshold = 1.0;
let leak = 0.1;
let input = 0.6;

// `elapsed_bursts` = bursts since the neuron last updated; V = V*(1-leak)^elapsed + input
let fired = update_neuron_lif(&mut potential, &mut fraction, threshold, 0.0, leak, 0.0, input, 1);
assert!(fired); // 0.5 * 0.9 + 0.6 = 1.05 >= 1.0
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


