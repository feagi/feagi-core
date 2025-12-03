# feagi-runtime-embedded

Embedded runtime adapter for FEAGI - ESP32, RTOS, and resource-constrained systems.

## Overview

Provides no_std implementations with:
- Fixed-size arrays (no heap allocation)
- Stack-only operations
- RTOS compatibility
- Deterministic execution

## Installation

```toml
[dependencies]
feagi-runtime-embedded = "2.0"
```

## Usage

```rust
#![no_std]
use feagi_runtime_embedded::EmbeddedRuntime;

// Configure for ESP32 with fixed capacity
let runtime = EmbeddedRuntime::new(1000, 5000);
```

## Platform Support

- ESP32 (WROOM, S3, C3)
- Arduino (Due, MKR, Nano 33 IoT)
- STM32 (F4, F7, H7 series)
- Teensy (4.0, 4.1)
- Any RTOS (FreeRTOS, Zephyr, bare-metal)

Part of the [FEAGI](https://github.com/feagi/feagi-core) ecosystem.

