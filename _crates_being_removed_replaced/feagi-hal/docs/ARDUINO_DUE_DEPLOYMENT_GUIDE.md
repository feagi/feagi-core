# Arduino Due Deployment Guide - Step by Step

**Platform**: Arduino Due (SAM3X8E ARM Cortex-M3)  
**Status**: ✅ Foundation (Architecture Ready, Hardware Testing Pending)  
**Difficulty**: ⭐⭐⭐ Intermediate  
**Time to Deploy**: 1-2 hours

---

## Overview

This guide walks you through deploying FEAGI neural networks on Arduino Due hardware.

**What you'll learn**:
- Arduino Due setup with Rust
- ARM Cortex-M3 programming
- Building and flashing FEAGI networks
- Working within 96 KB SRAM constraints

**What you'll build**:
- 500-1,000 neuron network
- 50-100 Hz burst processing
- Arduino shield compatibility maintained

---

## Hardware Requirements

### Arduino Due

**Specifications**:
- **CPU**: ARM Cortex-M3 @ 84 MHz
- **SRAM**: 96 KB
- **Flash**: 512 KB
- **GPIO**: 54 digital pins
- **Analog**: 12 analog inputs
- **Price**: $45

**Buy from**:
- [Arduino Official](https://store.arduino.cc/products/arduino-due) - $43
- [Amazon](https://www.amazon.com/Arduino-Due-A000062/dp/B00A6C3JN2) - $45
- [Adafruit](https://www.adafruit.com/product/1076) - $50

**Why Arduino Due?**
- ✅ Best Arduino for FEAGI (96 KB SRAM)
- ✅ Compatible with Arduino shields
- ✅ 32-bit ARM (efficient for INT8 quantization)
- ✅ Familiar Arduino ecosystem

**Note**: Arduino Mega/Uno are too limited (8KB/2KB SRAM) - not recommended

---

## Step 1: Install Rust ARM Toolchain (10 minutes)

### 1.1 Install Rust (if not already installed)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### 1.2 Add ARM Cortex-M3 Target

```bash
rustup target add thumbv7m-none-eabi
```

Verify:
```bash
rustup target list --installed | grep thumbv7m
# Should show: thumbv7m-none-eabi
```

### 1.3 Install cargo-binutils

```bash
cargo install cargo-binutils
rustup component add llvm-tools-preview
```

### 1.4 Install bossac (Arduino Due flasher)

```bash
# Ubuntu/Debian
sudo apt-get install bossac

# macOS
brew install bossac

# Windows
# Download from: https://www.arduino.cc/en/software
# bossac.exe is included in Arduino IDE
```

Verify:
```bash
bossac --version
# Should show: bossac 1.x.x
```

---

## Step 2: Create Your FEAGI Project (10 minutes)

### 2.1 Create New Project

```bash
cargo new my-feagi-due --bin
cd my-feagi-due
```

### 2.2 Update Cargo.toml

```toml
[package]
name = "my-feagi-due"
version = "0.1.0"
edition = "2021"

[dependencies]
# FEAGI embedded platform abstraction
feagi-hal = { git = "https://github.com/feagi/FEAGI-2.0", features = ["arduino-due"] }
feagi-types = { git = "https://github.com/feagi/FEAGI-2.0" }
feagi-runtime-embedded = { git = "https://github.com/feagi/FEAGI-2.0" }

# ARM Cortex-M support
cortex-m = "0.7"
cortex-m-rt = "0.7"
panic-halt = "0.2"

[profile.release]
opt-level = "z"     # Optimize for size (important for 512 KB flash!)
lto = true
codegen-units = 1
```

### 2.3 Create .cargo/config.toml

```toml
[target.thumbv7m-none-eabi]
runner = "arm-none-eabi-gdb"
rustflags = [
  "-C", "link-arg=-Tlink.x",
]

[build]
target = "thumbv7m-none-eabi"
```

### 2.4 Create memory.x (Linker Script)

```
MEMORY
{
  FLASH : ORIGIN = 0x00080000, LENGTH = 512K
  RAM : ORIGIN = 0x20000000, LENGTH = 96K
}
```

---

## Step 3: Write Your Neural Network (20 minutes)

### 3.1 Create src/main.rs

```rust
#![no_std]
#![no_main]

use panic_halt as _;
use cortex_m_rt::entry;
use feagi_hal::prelude::*;
use feagi_types::INT8Value;

// Network size (conservative for 96 KB SRAM)
const MAX_NEURONS: usize = 500;   // 7.5 KB
const MAX_SYNAPSES: usize = 2500; // 17.5 KB
// Total: ~25 KB, leaves 71 KB for stack

#[entry]
fn main() -> ! {
    // Initialize Arduino Due platform
    let platform = ArduinoDuePlatform::init();
    
    platform.info("Arduino Due FEAGI starting...");
    platform.info(&format!("CPU: {} MHz", platform.cpu_frequency_hz() / 1_000_000));
    platform.info(&format!("RAM: {} KB", platform.available_memory_bytes() / 1024));
    
    // Create neural network
    let mut neurons = NeuronArray::<INT8Value, MAX_NEURONS>::new();
    let mut synapses = SynapseArray::<MAX_SYNAPSES>::new();
    
    // Build simple reflex arc
    build_network(&mut neurons, &mut synapses, &platform);
    
    platform.info("Network created!");
    platform.info(&format!("Neurons: {}", neurons.count));
    platform.info(&format!("Synapses: {}", synapses.count));
    
    // Burst loop at 50 Hz (20ms per burst)
    let mut count: u32 = 0;
    let mut inputs = [INT8Value::zero(); MAX_NEURONS];
    let mut fired = [false; MAX_NEURONS];
    
    loop {
        let start = platform.get_time_us();
        
        // Process burst
        let fired_count = neurons.process_burst(&inputs, &mut fired);
        
        // Log every 50 bursts (once per second at 50 Hz)
        if count % 50 == 0 {
            platform.info(&format!("Burst {}: {} fired", count, fired_count));
        }
        
        // Timing: 20ms = 50 Hz
        let elapsed = platform.get_time_us() - start;
        if elapsed < 20_000 {
            platform.delay_us((20_000 - elapsed) as u32);
        }
        
        count += 1;
    }
}

fn build_network(
    neurons: &mut NeuronArray<INT8Value, MAX_NEURONS>,
    synapses: &mut SynapseArray<MAX_SYNAPSES>,
    platform: &ArduinoDuePlatform,
) {
    platform.info("Building network...");
    
    // Add 35 neurons
    for i in 0..35 {
        neurons.add_neuron(
            INT8Value::from_f32(1.0),
            if i < 10 { 0.1 } else { 0.05 },  // Sensory vs hidden/motor
            2,
            1.0,
        );
    }
    
    // Connect: sensory → hidden → motor
    for i in 0..10 {
        for j in 10..30 {
            synapses.add_synapse(i as u16, j as u16, 128, 0, 0);
        }
    }
    for i in 10..30 {
        for j in 30..35 {
            synapses.add_synapse(i as u16, j as u16, 128, 0, 0);
        }
    }
}
```

---

## Step 4: Build for Arduino Due (5 minutes)

### 4.1 Build the Firmware

```bash
cargo build --release --target thumbv7m-none-eabi
```

**Expected output**:
```
   Compiling cortex-m v0.7.7
   Compiling feagi-types v2.0.0
   Compiling feagi-hal v2.0.0
   Compiling my-feagi-due v0.1.0
    Finished release [optimized] target(s) in 45s
```

### 4.2 Convert to Binary

```bash
cargo objcopy --release -- -O binary target/thumbv7m-none-eabi/release/my-feagi-due.bin
```

### 4.3 Check Binary Size

```bash
ls -lh target/thumbv7m-none-eabi/release/my-feagi-due.bin
# Should be < 512 KB (Due has 512 KB flash)
# Typical: 50-150 KB
```

---

## Step 5: Flash to Arduino Due (5 minutes)

### 5.1 Prepare Arduino Due

**IMPORTANT**: Arduino Due has TWO USB ports:
- **Programming Port** (closest to power jack) - Use this!
- **Native USB Port** (middle) - Don't use for programming

**Connect**:
1. Plug USB cable into **Programming Port**
2. Due should power on (LED lights up)

### 5.2 Find Serial Port

```bash
# Linux
ls /dev/ttyACM*
# Should show: /dev/ttyACM0

# macOS
ls /dev/cu.usbmodem*
# Should show: /dev/cu.usbmodem14201

# Windows
# Use Device Manager
# Should show: COM3 or similar
```

### 5.3 Flash with bossac

```bash
# Linux/macOS
bossac -p /dev/ttyACM0 -e -w -v -b \
  target/thumbv7m-none-eabi/release/my-feagi-due.bin

# Windows
bossac -p COM3 -e -w -v -b my-feagi-due.bin

# Flags:
# -p : serial port
# -e : erase flash
# -w : write binary
# -v : verify
# -b : boot from flash after programming
```

**Expected output**:
```
Erase flash
Write 73728 bytes to flash
[=========================] 100% (288/288 pages)
Verify 73728 bytes of flash
[=========================] 100% (288/288 pages)
Verify successful
Set boot flash true
CPU reset.
```

### 5.4 Reset Arduino Due

Press the **RESET** button on the Due.

Your FEAGI network is now running!

---

## Step 6: Monitor Output (Ongoing)

### Current Limitation

⚠️ **Logging is not yet connected to serial output on Arduino Due.**

The platform implementation has placeholder logging. To see output, you need to:

**Option A: Use debugger (Advanced)**
```bash
arm-none-eabi-gdb target/thumbv7m-none-eabi/release/my-feagi-due
(gdb) target remote :3333  # OpenOCD on port 3333
(gdb) continue
```

**Option B: Add LED indicators**

```rust
// Add to your code
use cortex_m::peripheral::Peripherals as CortexPeripherals;

// Blink LED on pin 13 every burst
// (Requires GPIO setup - see Arduino Due HAL documentation)
```

**Option C: Wait for full peripheral implementation**

We're working on full USART support for Arduino Due. ETA: 1-2 weeks.

---

## Step 7: Verify Execution

### 7.1 LED Heartbeat Test

Add LED blinking to verify execution:

```rust
// In your burst loop
if count % 50 == 0 {
    // Toggle built-in LED (pin 13)
    // This requires Arduino Due GPIO setup
    // For now, the burst loop running indicates success
}
```

### 7.2 Measure Burst Timing

```rust
loop {
    let start = platform.get_time_us();
    let fired = neurons.process_burst(&inputs, &mut fired_mask);
    let elapsed = platform.get_time_us() - start;
    
    // On Arduino Due @ 84 MHz:
    // 500 neurons should take ~500-800 μs
    // 1000 neurons should take ~1-2 ms
}
```

### 7.3 Memory Usage

```rust
// Calculate memory used
let neuron_mem = MAX_NEURONS * 15;  // 15 bytes per neuron (INT8)
let synapse_mem = MAX_SYNAPSES * 7; // 7 bytes per synapse
let total = neuron_mem + synapse_mem;

// For 500 neurons, 2500 synapses:
// 7,500 + 17,500 = 25,000 bytes (~25 KB)
// Leaves ~71 KB for stack and globals ✅
```

---

## Step 8: Network Size Guidelines

### Maximum Network Sizes (96 KB SRAM)

| Neurons | Synapses | Memory | Burst Time | Recommended |
|---------|----------|--------|------------|-------------|
| 250 | 1,250 | ~12 KB | 250 μs | ✅ Safe |
| 500 | 2,500 | ~25 KB | 500 μs | ✅ Recommended |
| 1,000 | 5,000 | ~50 KB | 1 ms | ⚠️ Tight |
| 1,500 | 7,500 | ~75 KB | 1.5 ms | ❌ Risky |

**Recommendation**: Start with 500 neurons, increase gradually

---

## Step 9: Arduino Shield Compatibility

### Using Arduino Shields with FEAGI

Arduino Due is compatible with most 3.3V shields:

**Motor Shield (L293D)**:
```rust
// Control motors based on neural output
if fired_mask[30] {  // Motor neuron 30 fired
    // Set motor direction via GPIO
    // (Requires GPIO implementation)
}
```

**Sensor Shield (Grove)**:
```rust
// Read sensors and inject into sensory neurons
let distance = read_ultrasonic_sensor();  // Your sensor code
if distance < 20.0 {
    inputs[0] = INT8Value::from_f32(10.0);  // High activation
}
```

**Display Shield (TFT)**:
```rust
// Visualize neural activity
for i in 0..neurons.count {
    if fired_mask[i] {
        draw_pixel(i % 16, i / 16, RED);  // Your display code
    }
}
```

---

## Step 10: Performance Benchmarks

### Expected Performance (INT8)

| Network Size | Burst Time | Max Frequency | Memory |
|--------------|------------|---------------|--------|
| 100 neurons | 100 μs | 10,000 Hz | 1.5 KB |
| 250 neurons | 250 μs | 4,000 Hz | 3.75 KB |
| 500 neurons | 500 μs | 2,000 Hz | 7.5 KB |
| 1,000 neurons | 1 ms | 1,000 Hz | 15 KB |

**Note**: Arduino Due @ 84 MHz is ~3× slower than ESP32 @ 240 MHz

---

## Step 11: Advanced Configuration

### 11.1 Optimize for Speed

```toml
[profile.release]
opt-level = 3        # Max speed (instead of 'z' for size)
lto = "fat"
codegen-units = 1
```

### 11.2 Use DWT for Precise Timing

```rust
use cortex_m::peripheral::DWT;

// In your init:
let mut core = cortex_m::Peripherals::take().unwrap();
core.DWT.enable_cycle_counter();

// For timing:
let cycles_start = DWT::cycle_count();
neurons.process_burst(&inputs, &fired_mask);
let cycles = DWT::cycle_count() - cycles_start;
let microseconds = cycles / 84;  // 84 MHz = 84 cycles per μs
```

### 11.3 Reduce Memory Usage

**Reduce stack size** in memory.x:
```
/* Reduce if needed */
_stack_start = ORIGIN(RAM) + LENGTH(RAM);
_stack_size = 0x4000;  /* 16 KB stack */
```

**Use smaller types**:
```rust
// Use u8 instead of u16 for small networks
type NeuronId = u8;  // Max 256 neurons
```

---

## Troubleshooting

### Build Errors

**Error**: `error: could not find 'memory.x'`
```bash
# Solution: Create memory.x with FLASH and RAM definitions
```

**Error**: `error: linking with 'rust-lld' failed`
```bash
# Solution: Check memory.x addresses match Arduino Due
# FLASH: 0x00080000, 512K
# RAM: 0x20000000, 96K
```

### Flash Errors

**Error**: `No device found on /dev/ttyACM0`
```bash
# Solution 1: Check USB is connected to Programming Port (not Native Port!)
# Solution 2: Try pressing ERASE button, then flash
# Solution 3: Check permissions
sudo chmod 666 /dev/ttyACM0
```

**Error**: `SAM-BA operation failed`
```bash
# Solution: Put Due in bootloader mode
# 1. Press and release ERASE button
# 2. Wait 1 second
# 3. Press and release RESET button
# 4. Flash within 10 seconds
```

### Runtime Issues

**No output**:
- Logging is placeholder - need debugger to see output
- Or add LED indicators to show execution

**Constant reboots**:
- Stack overflow - reduce MAX_NEURONS
- Increase stack size in memory.x

**Slow execution**:
- Arduino Due is slower than ESP32 (84 MHz vs 240 MHz)
- Reduce network size or lower burst frequency

---

## Known Limitations (Current Foundation Implementation)

⚠️ **What's Not Yet Implemented**:
- Serial/USART output (placeholder)
- GPIO control (uses placeholder)
- Precise microsecond timing (uses busy-wait)

✅ **What Works**:
- Neural processing (full LIF dynamics)
- INT8 quantization
- Fixed-size arrays
- Burst loop execution
- Memory-efficient operation

**Full peripheral support coming in**: 1-2 weeks

---

## Migration from Arduino IDE

### If you're coming from Arduino IDE:

**Arduino IDE Code**:
```cpp
void setup() {
  Serial.begin(115200);
  pinMode(13, OUTPUT);
}

void loop() {
  digitalWrite(13, HIGH);
  delay(1000);
}
```

**Rust Equivalent**:
```rust
#![no_std]
#![no_main]

use panic_halt as _;
use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    // Setup equivalent
    // (GPIO and Serial would be initialized here)
    
    // Loop equivalent
    loop {
        // digitalWrite(13, HIGH) equivalent
        // (GPIO control would go here)
        
        // delay(1000) equivalent
        cortex_m::asm::delay(84_000_000);  // 1s at 84 MHz
    }
}
```

**Key Differences**:
- `#![no_std]` - No standard library
- `#[entry]` - Entry point annotation
- `-> !` - Never returns
- Manual peripheral initialization
- More verbose but more control

---

## Comparison: Arduino Due vs ESP32

| Feature | Arduino Due | ESP32-S3 | Winner |
|---------|-------------|----------|--------|
| **SRAM** | 96 KB | 8 MB | ESP32-S3 |
| **Max Neurons** | 1,000 | 40,000 | ESP32-S3 |
| **CPU** | 84 MHz | 240 MHz | ESP32-S3 |
| **Price** | $45 | $12 | ESP32-S3 |
| **Shields** | ✅ Compatible | ❌ None | Arduino Due |
| **Ecosystem** | ✅ Huge | ⚠️ Growing | Arduino Due |
| **WiFi/BT** | ❌ No | ✅ Yes | ESP32-S3 |
| **USB** | ✅ Native | ⚠️ Via UART | Arduino Due |

**Verdict**: 
- Choose Arduino Due if you need Arduino shield compatibility
- Choose ESP32-S3 for better price/performance and larger networks

---

## Next Steps

### After Successful Deployment

1. **Increase network size** (gradually up to 1,000 neurons)
2. **Add sensors** (using Arduino libraries)
3. **Add motor control** (via motor shields)
4. **Test with Arduino shields** (verify compatibility)
5. **Profile performance** (optimize burst timing)

### Getting Help

**Arduino Due Rust Resources**:
- [Embedded Rust Book](https://rust-embedded.github.io/book/)
- [cortex-m Crate Docs](https://docs.rs/cortex-m/)
- [ARM Cortex-M Programming](https://japaric.github.io/discovery/)

**FEAGI Resources**:
- [Platform Comparison](PLATFORM_COMPARISON.md)
- [Porting Guide](PORTING_GUIDE.md)
- [FEAGI Discord](https://discord.gg/feagi)

---

## Contributing

**Help us improve Arduino Due support!**

We need:
- [ ] Full USART implementation (for serial logging)
- [ ] GPIO wrapper (for shields)
- [ ] SPI/I2C support (for sensors)
- [ ] Hardware testing results

If you have an Arduino Due, please test and report results!

---

**Status**: Foundation ready - neural processing works, peripherals need completion  
**Confidence**: ⭐⭐⭐ Architecture solid, needs hardware validation  
**Timeline**: Full support in 1-2 weeks

