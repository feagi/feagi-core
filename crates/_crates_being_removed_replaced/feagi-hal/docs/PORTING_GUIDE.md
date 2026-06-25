# Platform Porting Guide

**Date**: November 4, 2025  
**Audience**: Platform maintainers and contributors

---

## Overview

This guide shows how to add a new platform to `feagi-hal` in 2-3 days.

**Effort Estimate**:
- HAL trait implementation: 4-6 hours
- Peripheral setup: 4-8 hours
- Testing: 8-16 hours
- Documentation: 2-4 hours
- **Total**: 2-3 days

---

## Step 1: Create Platform Module (30 minutes)

Create `src/platforms/your_platform.rs`:

```rust
/// Your Platform implementation
use crate::hal::*;

#[cfg(feature = "your-platform")]
pub struct YourPlatform {
    // Store platform-specific state
    start_time_us: u64,
}

#[cfg(feature = "your-platform")]
impl YourPlatform {
    pub fn init() -> Result<Self, &'static str> {
        // Initialize hardware
        // - Clock configuration
        // - Peripheral initialization
        // - Timer setup
        Ok(Self {
            start_time_us: 0,
        })
    }
    
    pub fn chip_model(&self) -> &'static str {
        "Your Chip Model"
    }
}

// Placeholder for when feature is not enabled
#[cfg(not(feature = "your-platform"))]
pub struct YourPlatform;

#[cfg(not(feature = "your-platform"))]
impl YourPlatform {
    pub fn init() -> Result<Self, &'static str> {
        Err("Platform feature not enabled. Rebuild with --features your-platform")
    }
}
```

---

## Step 2: Implement TimeProvider Trait (1-2 hours)

```rust
#[cfg(feature = "your-platform")]
impl TimeProvider for YourPlatform {
    fn get_time_us(&self) -> u64 {
        // Option A: Use hardware timer
        // - Read timer counter register
        // - Convert to microseconds
        
        // Option B: Use DWT cycle counter (ARM Cortex-M)
        // unsafe { (*cortex_m::peripheral::DWT::PTR).cyccnt as u64 / CPU_MHZ }
        
        // Option C: Use system tick
        // - Less precise but works everywhere
        
        self.start_time_us  // Placeholder
    }
    
    fn delay_us(&self, us: u32) {
        // Option A: Hardware timer delay
        // - Configure timer for us delay
        // - Wait for interrupt
        
        // Option B: Busy-wait with cycle counting
        let cycles = (us as u64 * CPU_FREQUENCY_HZ) / 1_000_000;
        for _ in 0..cycles {
            cortex_m::asm::nop();
        }
    }
}
```

**Tips**:
- Use hardware timer for best precision
- DWT cycle counter works well on ARM Cortex-M
- Busy-wait is acceptable for short delays (<1ms)

---

## Step 3: Implement SerialIO Trait (2-3 hours)

```rust
#[cfg(feature = "your-platform")]
impl SerialIO for YourPlatform {
    type Error = ();  // Or your platform's error type
    
    fn write(&mut self, data: &[u8]) -> Result<usize, Self::Error> {
        // Write to UART/USART peripheral
        // Example for typical UART:
        // for &byte in data {
        //     while !uart.is_tx_ready() {}
        //     uart.write_byte(byte);
        // }
        Ok(data.len())
    }
    
    fn read(&mut self, buffer: &mut [u8]) -> Result<usize, Self::Error> {
        // Non-blocking read from UART
        // let mut count = 0;
        // for slot in buffer.iter_mut() {
        //     if uart.is_rx_ready() {
        //         *slot = uart.read_byte();
        //         count += 1;
        //     } else {
        //         break;
        //     }
        // }
        // Ok(count)
        Ok(0)  // Placeholder
    }
    
    fn flush(&mut self) -> Result<(), Self::Error> {
        // Wait for TX complete
        // while !uart.is_tx_complete() {}
        Ok(())
    }
}
```

---

## Step 4: Implement Logger Trait (1 hour)

```rust
#[cfg(feature = "your-platform")]
impl Logger for YourPlatform {
    fn log(&self, _level: LogLevel, _message: &str) {
        // Option A: Write to USART (no_std)
        // - Use ufmt or core::fmt::Write
        // - Output to serial console
        
        // Option B: Use ITM (ARM Cortex-M)
        // - Instrumentation Trace Macrocell
        // - Viewable in debugger
        
        // Option C: Use semihosting (debug only)
        // - cortex_m_semihosting::hprintln!
        
        // For now: no-op (will implement when peripherals are ready)
    }
}
```

---

## Step 5: Implement Platform Trait (30 minutes)

```rust
#[cfg(feature = "your-platform")]
impl Platform for YourPlatform {
    fn name(&self) -> &'static str {
        self.chip_model()
    }
    
    fn cpu_frequency_hz(&self) -> u32 {
        // Return actual CPU frequency
        // Could read from clock configuration registers
        168_000_000  // Example: 168 MHz
    }
    
    fn available_memory_bytes(&self) -> usize {
        // Option A: Total SRAM size (if no malloc)
        // Option B: Query heap (if using allocator)
        // Option C: Calculate free stack space
        
        192_000  // Example: 192 KB total SRAM
    }
}
```

---

## Step 6: Update Cargo.toml (15 minutes)

Add dependencies:

```toml
[dependencies]
# Your platform HAL
your-platform-hal = { version = "1.0", optional = true }
cortex-m = { version = "0.7", optional = true }
cortex-m-rt = { version = "0.7", optional = true }
```

Add feature:

```toml
[features]
your-platform = ["your-platform-hal", "embedded-hal", "cortex-m", "cortex-m-rt"]
```

---

## Step 7: Update Module Exports (10 minutes)

In `src/platforms/mod.rs`:

```rust
#[cfg(feature = "your-platform")]
pub mod your_platform;

#[cfg(feature = "your-platform")]
pub use your_platform::YourPlatform;
```

In `src/lib.rs`:

```rust
#[cfg(feature = "your-platform")]
pub use platforms::YourPlatform;

// In prelude:
pub mod prelude {
    // ...
    #[cfg(feature = "your-platform")]
    pub use crate::platforms::YourPlatform;
}
```

---

## Step 8: Test Compilation (30 minutes)

```bash
# Test basic compilation
cargo check --no-default-features --features your-platform

# Test for correct target
cargo build --release --features your-platform --target your-target-triple

# Test with all ARM platforms
cargo check --features all-arm-cortex-m
```

---

## Step 9: Hardware Testing (1-2 days)

Create a test application:

```rust
use feagi_hal::prelude::*;
use feagi_runtime_embedded::{NeuronArray, SynapseArray};
use feagi_types::INT8Value;

#[no_std]
#[no_main]
fn main() -> ! {
    let platform = YourPlatform::init().expect("Init failed");
    
    platform.info("FEAGI test starting...");
    
    let mut neurons = NeuronArray::<INT8Value, 100>::new();
    let mut synapses = SynapseArray::<500>::new();
    
    // Add test neurons
    for _ in 0..10 {
        neurons.add_neuron(
            INT8Value::from_f32(1.0),
            0.1,
            2,
            1.0,
        );
    }
    
    // Burst loop
    let mut count = 0;
    loop {
        let start = platform.get_time_us();
        neurons.process_burst(&synapses);
        let elapsed = platform.get_time_us() - start;
        
        if count % 100 == 0 {
            platform.info(&format!("Burst {} - {}μs", count, elapsed));
        }
        
        platform.delay_ms(10);  // 100 Hz
        count += 1;
    }
}
```

**Validation checklist**:
- [ ] Platform initializes correctly
- [ ] Timing functions work (get_time_us, delay_us)
- [ ] Logging works (messages visible)
- [ ] Burst loop runs at target frequency
- [ ] Memory usage is as expected
- [ ] No crashes or panics

---

## Step 10: Documentation (2-4 hours)

Update:
- [ ] `README.md` - Add platform to supported list
- [ ] `docs/PLATFORM_COMPARISON.md` - Add specs and capabilities
- [ ] `docs/PORTING_GUIDE.md` - Add lessons learned
- [ ] `src/platforms/your_platform.rs` - Add comprehensive rustdoc comments

---

## Common Patterns

### Pattern 1: ARM Cortex-M Platforms

Most ARM platforms share:
- DWT cycle counter for timing
- NVIC for interrupts
- Standard peripheral interfaces (USART, GPIO, SPI, I2C)

**Example**: Arduino Due, STM32F4, Raspberry Pi Pico

### Pattern 2: ESP32 Family

ESP32 platforms use:
- ESP-IDF framework
- FreeRTOS under the hood
- esp_timer for microsecond timing
- esp-idf-hal for peripherals

**Example**: ESP32, ESP32-S3, ESP32-C3

### Pattern 3: RISC-V Platforms

RISC-V platforms vary more:
- Standard timer (mtime/mtimecmp)
- Platform-specific peripherals
- May or may not have FPU

**Example**: ESP32-C3, SiFive boards

---

## Tips for Success

### ✅ DO:
- Start with TimeProvider (easiest to test)
- Use existing platform-hal crates when available
- Test compilation frequently
- Document limitations clearly
- Ask questions in feagi-hal issues

### ❌ DON'T:
- Try to implement everything at once
- Use std library (must be no_std)
- Use format! macro (use ufmt instead)
- Allocate dynamically (use fixed arrays)
- Skip hardware testing

---

## Example: ESP32 Platform (Reference Implementation)

See `src/platforms/esp32.rs` for a complete, production-ready implementation:

```1:140:feagi-core/crates/feagi-hal/src/platforms/esp32.rs
/// ESP32 platform implementation
/// 
/// Supports ESP32, ESP32-S3, ESP32-C3 (RISC-V) variants

use crate::hal::*;

#[cfg(feature = "esp32")]
use esp_idf_svc::hal::{
    peripherals::Peripherals,
    uart::{config::Config as UartConfig, UartDriver},
};
#[cfg(feature = "esp32")]
use esp_idf_svc::sys as esp_idf_sys;

/// ESP32 platform structure
#[cfg(feature = "esp32")]
pub struct Esp32Platform {
    uart: Option<UartDriver<'static>>,
}
// ... (see full file for details)
```

This implementation shows:
- Proper peripheral handling
- Conditional compilation
- Error handling
- Documentation

---

## Getting Help

- **GitHub Issues**: https://github.com/feagi/feagi-2.0/issues
- **Discussions**: https://github.com/feagi/feagi-2.0/discussions
- **Discord**: https://discord.gg/feagi

Tag issues with `platform-support` for platform-specific questions.

---

## Conclusion

**Adding a new platform is straightforward with this architecture!**

The trait-based design ensures:
- ✅ Clear contract (HAL traits)
- ✅ Type safety (compile-time checks)
- ✅ Zero overhead (monomorphization)
- ✅ Isolated platform code (~100-200 lines per platform)

**Total effort**: 2-3 days from start to hardware-tested platform 🚀

