# feagi-hal

**Hardware Abstraction Layer (HAL) for FEAGI embedded neural networks**

Platform abstraction and implementations for embedded systems.

Part of [FEAGI 2.0](https://github.com/feagi/feagi) - Framework for Evolutionary AGI

---

## Overview

`feagi-hal` provides a Hardware Abstraction Layer (HAL) with platform-agnostic traits and concrete implementations for running FEAGI neural networks on embedded systems. It sits between the platform-agnostic neural processing core and platform-specific hardware.

### Architecture

```
┌─────────────────────────────────────────────────────────┐
│  Application (feagi-nano SDK or custom application)    │
└─────────────────────────────────────────────────────────┘
                        ↓ uses
┌─────────────────────────────────────────────────────────┐
│  feagi-hal (THIS CRATE)                            │
│  ├── hal/         Platform Abstraction Layer (traits)   │
│  └── platforms/   Platform Implementations              │
└─────────────────────────────────────────────────────────┘
                        ↓ uses
┌─────────────────────────────────────────────────────────┐
│  feagi-core (neural processing)                         │
│  ├── feagi-types                                        │
│  ├── feagi-neural                                       │
│  ├── feagi-synapse                                      │
│  └── feagi-runtime-embedded                             │
└─────────────────────────────────────────────────────────┘
```

---

## Features

### Platform Abstraction Layer (HAL)

- **TimeProvider** - Monotonic time and delays
- **SerialIO** - UART/serial communication
- **GpioProvider** - Digital I/O control
- **Logger** - Structured logging
- **NeuralAccelerator** - Hardware acceleration (Hailo, TPU, etc.)

### Supported Platforms

| Platform | Status | Feature Flag | Target | Max Neurons (INT8) |
|----------|--------|--------------|--------|--------------------|
| ESP32 | ✅ Production | `esp32` | `xtensa-esp32-none-elf` | 2,000 |
| ESP32-S3 | ✅ Production | `esp32-s3` | `xtensa-esp32s3-none-elf` | 40,000 |
| ESP32-C3 | ✅ Production | `esp32-c3` | `riscv32imc-esp-espidf` | 1,500 |
| Arduino Due | ✅ Foundation | `arduino-due` | `thumbv7m-none-eabi` | 1,000 |
| STM32F4 | ✅ Foundation | `stm32f4` | `thumbv7em-none-eabihf` | 2,500 |
| Raspberry Pi Pico | ✅ Foundation | `rpi-pico` | `thumbv6m-none-eabi` | 3,500 |
| **Hailo-8** | ✅ **Foundation** | `hailo` | `aarch64-unknown-linux-gnu` | **1,000,000+** 🚀 |

**Legend**:
- ✅ **Production**: Fully tested on hardware, production-ready (ESP32 family)
- ✅ **Foundation**: Traits implemented, compiles successfully, needs hardware/FFI testing
- 🔧 **Planned**: Architecture designed, implementation pending

**Note**: Hailo-8 requires HailoRT C/C++ library and FFI bindings for hardware deployment. Architecture is complete!

---

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
feagi-hal = { version = "2.0", features = ["esp32"] }
```

### Feature Flags

```toml
# Microcontrollers
esp32 = ["esp-idf-svc", "esp-idf-hal"]           # ESP32 family
arduino-due = ["arduino-hal"]                     # Arduino Due (future)
stm32f4 = ["stm32f4xx-hal"]                      # STM32F4 series (future)

# Neural accelerators
hailo = ["hailo-sdk"]                            # Hailo-8 (future)

# Convenience bundles
all-esp32 = ["esp32", "esp32-s3", "esp32-c3"]
```

---

## Usage

### Quick Start (ESP32)

```rust
use feagi_hal::prelude::*;

fn main() -> ! {
    // Initialize platform
    let platform = Esp32Platform::init().expect("Failed to initialize ESP32");
    
    platform.info("FEAGI Embedded starting...");
    platform.info(&format!("Platform: {}", platform.name()));
    platform.info(&format!("CPU: {} MHz", platform.cpu_frequency_hz() / 1_000_000));
    
    // Your neural network code here
    let mut neurons = NeuronArray::<INT8Value, 1000>::new();
    let mut synapses = SynapseArray::<5000>::new();
    
    loop {
        let start = platform.get_time_us();
        
        // Process neural burst
        neurons.process_burst(&synapses);
        
        // Timing control
        let elapsed = platform.get_time_us() - start;
        if elapsed < 10_000 { // 10ms = 100 Hz
            platform.delay_us((10_000 - elapsed) as u32);
        }
    }
}
```

### Build for ESP32

```bash
# Install ESP32 toolchain
cargo install espup
espup install
source ~/export-esp.sh

# Build
cargo build --release --features esp32 --target xtensa-esp32-none-elf

# Flash
cargo run --release --features esp32
```

---

## HAL Trait Definitions

### TimeProvider

```rust
pub trait TimeProvider {
    fn get_time_us(&self) -> u64;
    fn delay_us(&self, us: u32);
    fn delay_ms(&self, ms: u32);
}
```

### SerialIO

```rust
pub trait SerialIO {
    type Error;
    fn write(&mut self, data: &[u8]) -> Result<usize, Self::Error>;
    fn read(&mut self, buffer: &mut [u8]) -> Result<usize, Self::Error>;
    fn flush(&mut self) -> Result<(), Self::Error>;
}
```

### Logger

```rust
pub trait Logger {
    fn log(&self, level: LogLevel, message: &str);
    fn error(&self, message: &str);
    fn warn(&self, message: &str);
    fn info(&self, message: &str);
}
```

### NeuralAccelerator

```rust
pub trait NeuralAccelerator {
    type Error;
    fn is_available(&self) -> bool;
    fn upload_neurons(&mut self, neurons: &[u8]) -> Result<(), Self::Error>;
    fn process_burst(&mut self) -> Result<u32, Self::Error>;
    fn download_neurons(&mut self, buffer: &mut [u8]) -> Result<usize, Self::Error>;
}
```

---

## Adding a New Platform

See [PORTING_GUIDE.md](docs/PORTING_GUIDE.md) for step-by-step instructions.

**Quick overview:**

1. Create `src/platforms/myplatform.rs`
2. Implement HAL traits (`TimeProvider`, `SerialIO`, `Logger`, `Platform`)
3. Add feature flag to `Cargo.toml`
4. Add platform-specific HAL dependencies
5. Test on hardware

**Estimated time**: 2-3 days per platform

---

## Documentation

- [API Documentation](https://docs.rs/feagi-hal)
- [Porting Guide](docs/PORTING_GUIDE.md)
- [Platform Comparison](docs/PLATFORM_COMPARISON.md)
- [feagi-nano SDK](../../../feagi-nano/README.md) - High-level application framework

---

## Who Uses This Crate?

### Direct Users (Advanced)
- Researchers needing full control over neural network execution
- Product companies integrating FEAGI into existing embedded systems
- Custom neural network topologies

### Indirect Users (via feagi-nano SDK)
- Embedded developers using NetworkBuilder API
- Robotics engineers using pre-built templates
- Students using ready-to-use binaries

---

## Architecture Notes

### Why Separate from feagi-nano?

- **feagi-hal**: Low-level, stable API for platform abstraction
- **feagi-nano**: High-level SDK with NetworkBuilder, templates, etc.

This separation allows:
- Other projects to use `feagi-hal` directly
- Independent evolution of SDK features
- Reusable platform implementations

### Design Principles

1. **Zero-cost abstractions** - Traits compile to direct function calls
2. **Platform-agnostic core** - Neural processing code works everywhere
3. **Minimal dependencies** - Only platform HALs, no std
4. **Type-safe** - Compile-time checks for platform compatibility

---

## License

Licensed under Apache License 2.0

Copyright © 2025 Neuraville Inc.

---

## Contributing

See [CONTRIBUTING.md](../../../CONTRIBUTING.md)

**Platform implementations welcome!** We're looking for contributors to add support for:
- Arduino Due, Mega
- STM32F4, STM32H7
- Raspberry Pi Pico
- Nordic nRF52
- Hailo-8, Google Coral TPU

---

## Links

- [FEAGI Project](https://github.com/feagi/feagi)
- [feagi-nano SDK](../../../feagi-nano/)
- [Documentation](https://docs.feagi.org)

