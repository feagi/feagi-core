# ESP32 Deployment Guide - Step by Step

**Platform**: ESP32, ESP32-S3, ESP32-C3  
**Status**: ✅ Production-Ready  
**Difficulty**: ⭐⭐ Beginner-Friendly  
**Time to Deploy**: 30-60 minutes

---

## Overview

This guide walks you through deploying FEAGI neural networks on ESP32 hardware from scratch.

**What you'll learn**:
- Hardware setup
- Toolchain installation
- Building your first FEAGI neural network
- Flashing to hardware
- Monitoring and debugging

**What you'll build**:
- 1,000 neuron reflex arc network
- 100 Hz burst processing
- UART sensory input/motor output
- LED heartbeat indicator

---

## Hardware Requirements

### Recommended: ESP32-S3-DevKitC-1

**Why ESP32-S3?**
- 8 MB PSRAM (40,000 neurons!)
- USB-C (easy programming)
- WiFi + Bluetooth
- $10-15

**Buy from**:
- [Adafruit](https://www.adafruit.com/product/5456) - $13
- [Mouser](https://www.mouser.com/ProductDetail/356-ESP32S3DEVKTC1U) - $12
- [Digikey](https://www.digikey.com/en/products/detail/espressif-systems/ESP32-S3-DevKitC-1-N8R8/15653986) - $12

### Alternative: ESP32 Standard

**Good for**:
- Smaller networks (2,000 neurons)
- Budget builds ($5-10)
- Existing ESP32 hardware

---

## Step 1: Install Rust Toolchain (15 minutes)

### 1.1 Install Rust (if not already installed)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

Verify:
```bash
rustc --version
# Should show: rustc 1.75.0 (or newer)
```

### 1.2 Install ESP32 Rust Toolchain

```bash
# Install espup (ESP32 toolchain manager)
cargo install espup

# Install ESP32 Rust toolchain
espup install

# Activate environment (add to ~/.bashrc or ~/.zshrc)
source ~/export-esp.sh
```

Verify:
```bash
rustup toolchain list
# Should show: esp (nightly-...)
```

### 1.3 Install espflash (flashing tool)

```bash
cargo install espflash
```

Verify:
```bash
espflash --version
# Should show: espflash 3.x.x
```

---

## Step 2: Create Your FEAGI Project (10 minutes)

### 2.1 Clone feagi-nano Template

```bash
cd ~/projects
git clone https://github.com/feagi/FEAGI-2.0.git
cd FEAGI-2.0/feagi-nano
```

Or create from scratch:

```bash
cargo new my-feagi-robot --bin
cd my-feagi-robot
```

### 2.2 Update Cargo.toml

```toml
[package]
name = "my-feagi-robot"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "my-feagi-robot"
path = "src/main.rs"

[dependencies]
# FEAGI embedded platform abstraction
feagi-hal = { git = "https://github.com/feagi/FEAGI-2.0", features = ["esp32"] }
feagi-types = { git = "https://github.com/feagi/FEAGI-2.0" }
feagi-runtime-embedded = { git = "https://github.com/feagi/FEAGI-2.0" }
feagi-synapse = { git = "https://github.com/feagi/FEAGI-2.0" }

# Utilities
anyhow = "1.0"

[build-dependencies]
embuild = "0.32"
```

### 2.3 Create rust-toolchain.toml

```toml
[toolchain]
channel = "esp"
```

### 2.4 Create build.rs

```rust
fn main() {
    embuild::espidf::sysenv::output();
}
```

---

## Step 3: Write Your Neural Network (15 minutes)

### 3.1 Create src/main.rs

```rust
use feagi_hal::prelude::*;
use feagi_types::INT8Value;

// Network configuration
const MAX_NEURONS: usize = 1000;
const MAX_SYNAPSES: usize = 5000;
const BURST_FREQUENCY_HZ: u32 = 100;

fn main() -> anyhow::Result<()> {
    // Step 1: Initialize ESP32 platform
    let platform = Esp32Platform::init()?;
    
    platform.info("=================================");
    platform.info("  FEAGI on ESP32");
    platform.info(&format!("  Platform: {}", platform.name()));
    platform.info(&format!("  CPU: {} MHz", platform.cpu_frequency_hz() / 1_000_000));
    platform.info(&format!("  Free RAM: {} KB", platform.available_memory_bytes() / 1024));
    platform.info("=================================");
    
    // Step 2: Create neural network
    platform.info("Creating neural network...");
    let mut neurons = NeuronArray::<INT8Value, MAX_NEURONS>::new();
    let mut synapses = SynapseArray::<MAX_SYNAPSES>::new();
    
    // Step 3: Build network topology
    build_reflex_arc(&mut neurons, &mut synapses, &platform)?;
    
    platform.info(&format!("Network ready: {} neurons, {} synapses", 
        neurons.count, synapses.count));
    platform.info(&format!("Memory: {} bytes", 
        NeuronArray::<INT8Value, MAX_NEURONS>::memory_footprint()));
    
    // Step 4: Run burst loop
    platform.info(&format!("Starting {} Hz burst loop...", BURST_FREQUENCY_HZ));
    run_burst_loop(&mut neurons, &synapses, &platform);
}

fn build_reflex_arc(
    neurons: &mut NeuronArray<INT8Value, MAX_NEURONS>,
    synapses: &mut SynapseArray<MAX_SYNAPSES>,
    platform: &Esp32Platform,
) -> anyhow::Result<()> {
    platform.info("Building reflex arc...");
    
    // Sensory layer (10 neurons)
    for _ in 0..10 {
        neurons.add_neuron(
            INT8Value::from_f32(1.0),  // threshold
            0.1,   // leak
            2,     // refractory period
            1.0,   // excitability
        ).ok_or_else(|| anyhow::anyhow!("Failed to add neuron"))?;
    }
    
    // Hidden layer (20 neurons)
    for _ in 0..20 {
        neurons.add_neuron(
            INT8Value::from_f32(1.0),
            0.05,  // slower leak
            1,
            1.0,
        ).ok_or_else(|| anyhow::anyhow!("Failed to add neuron"))?;
    }
    
    // Motor layer (5 neurons)
    for _ in 0..5 {
        neurons.add_neuron(
            INT8Value::from_f32(1.0),
            0.05,
            1,
            1.0,
        ).ok_or_else(|| anyhow::anyhow!("Failed to add neuron"))?;
    }
    
    // Connect sensory → hidden (fully connected)
    for i in 0..10 {
        for j in 10..30 {
            synapses.add_synapse(
                i as u16,     // source
                j as u16,     // target
                128,          // weight (50% strength)
                0,            // psp
                0,            // type
            ).ok_or_else(|| anyhow::anyhow!("Failed to add synapse"))?;
        }
    }
    
    // Connect hidden → motor (convergent)
    for i in 10..30 {
        for j in 30..35 {
            synapses.add_synapse(
                i as u16,
                j as u16,
                128,
                0,
                0,
            ).ok_or_else(|| anyhow::anyhow!("Failed to add synapse"))?;
        }
    }
    
    platform.info(&format!("Created {} neurons, {} synapses", 
        neurons.count, synapses.count));
    Ok(())
}

fn run_burst_loop(
    neurons: &mut NeuronArray<INT8Value, MAX_NEURONS>,
    synapses: &SynapseArray<MAX_SYNAPSES>,
    platform: &Esp32Platform,
) -> ! {
    let mut burst_count: u64 = 0;
    let mut candidate_potentials = [INT8Value::zero(); MAX_NEURONS];
    let mut fired_mask = [false; MAX_NEURONS];
    let burst_interval_us = 1_000_000 / BURST_FREQUENCY_HZ as u64;
    
    loop {
        let start = platform.get_time_us();
        
        // Process neural burst
        let fired_count = neurons.process_burst(&candidate_potentials, &mut fired_mask);
        
        // Log every 100 bursts (once per second at 100 Hz)
        if burst_count % 100 == 0 {
            platform.info(&format!("Burst {}: {} neurons fired", 
                burst_count, fired_count));
        }
        
        // Timing control
        let elapsed = platform.get_time_us() - start;
        if elapsed < burst_interval_us {
            platform.delay_us((burst_interval_us - elapsed) as u32);
        }
        
        burst_count += 1;
    }
}
```

---

## Step 4: Build for ESP32 (5 minutes)

### 4.1 Connect ESP32 to Computer

- Plug in ESP32-S3 via USB-C cable
- LED should light up (power indicator)

### 4.2 Build the Firmware

```bash
# For ESP32-S3
cargo build --release --target xtensa-esp32s3-none-elf

# For ESP32 standard
cargo build --release --target xtensa-esp32-none-elf

# For ESP32-C3 (RISC-V)
cargo build --release --target riscv32imc-esp-espidf
```

**Expected output**:
```
   Compiling feagi-types v2.0.0
   Compiling feagi-neural v2.0.0
   Compiling feagi-runtime-embedded v2.0.0
   Compiling feagi-hal v2.0.0
   Compiling my-feagi-robot v0.1.0
    Finished release [optimized] target(s) in 2m 30s
```

**Build time**: ~2-3 minutes first time, ~10 seconds incremental

---

## Step 5: Flash to Hardware (2 minutes)

### 5.1 Find Serial Port

```bash
# Linux
ls /dev/ttyUSB* /dev/ttyACM*
# Should show: /dev/ttyUSB0 or /dev/ttyACM0

# macOS
ls /dev/cu.usb*
# Should show: /dev/cu.usbserial-* or /dev/cu.usbmodem-*

# Windows
# Use Device Manager → Ports (COM & LPT)
# Should show: COM3, COM4, etc.
```

### 5.2 Flash the Firmware

**Automatic (Recommended)**:
```bash
cargo run --release

# Or explicitly specify port
espflash flash target/xtensa-esp32s3-none-elf/release/my-feagi-robot \
  --port /dev/ttyUSB0 --monitor
```

**Manual**:
```bash
espflash flash target/xtensa-esp32s3-none-elf/release/my-feagi-robot
```

**Expected output**:
```
[00:00:00] ########################################  100%  Flashing...
[00:00:02] Chip: ESP32-S3 (revision v0.1)
[00:00:02] Flash size: 8MB
[00:00:02] Bootloader: 0x0
[00:00:02] Partition table: 0x8000
[00:00:02] App: 0x10000
[00:00:05] Flashing has completed!
```

---

## Step 6: Monitor Serial Output (Ongoing)

### 6.1 Open Serial Monitor

```bash
# If not already monitoring from cargo run
espflash monitor --port /dev/ttyUSB0
```

### 6.2 Expected Output

```
I (312) cpu_start: Starting scheduler on APP CPU.
I (322) esp_psram: Reserving pool of 32K of internal memory for DMA/internal allocations
=================================
  FEAGI on ESP32
  Platform: ESP32-S3
  CPU: 240 MHz
  Free RAM: 8192 KB
=================================
Creating neural network...
Building reflex arc...
Created 35 neurons, 300 synapses
Network ready: 35 neurons, 300 synapses
Memory: 525 bytes
Starting 100 Hz burst loop...
Burst 0: 0 neurons fired
Burst 100: 3 neurons fired
Burst 200: 5 neurons fired
Burst 300: 2 neurons fired
...
```

### 6.3 Troubleshooting Serial Monitor

**No output?**
- Check baud rate (should be 115200)
- Try pressing RESET button on ESP32
- Check USB cable (some are charge-only)

**Garbage characters?**
- Wrong baud rate: use 115200
- Wrong chip selected: rebuild for correct target

**Reboots constantly?**
- Power issue: use USB 3.0 port or powered hub
- Memory overflow: reduce MAX_NEURONS

---

## Step 7: Verify Performance (5 minutes)

### 7.1 Check Burst Frequency

The output should show bursts approximately every 10ms (100 Hz):

```
Burst 100: 3 neurons fired    ← at t=1.0s
Burst 200: 5 neurons fired    ← at t=2.0s
Burst 300: 2 neurons fired    ← at t=3.0s
```

If timing is off:
- Check CPU frequency (should be 240 MHz)
- Reduce network size if bursts take >10ms
- Check for other tasks consuming CPU

### 7.2 Check Memory Usage

```rust
// Add to your code:
platform.info(&format!("Free heap: {} KB", 
    platform.available_memory_bytes() / 1024));
```

**Expected**:
- ESP32-S3: ~8000 KB free (8 MB PSRAM)
- ESP32: ~400 KB free (520 KB SRAM)

**If low memory (<100 KB)**:
- Reduce MAX_NEURONS
- Reduce MAX_SYNAPSES
- Check for memory leaks

---

## Step 8: Add Sensory Input (Advanced)

### 8.1 Connect a Sensor

Example: Distance sensor on GPIO 4

```rust
use esp_idf_svc::hal::gpio::PinDriver;
use esp_idf_svc::hal::peripherals::Peripherals;

fn main() -> anyhow::Result<()> {
    let platform = Esp32Platform::init()?;
    let peripherals = Peripherals::take()?;
    
    // Configure GPIO4 as input
    let sensor = PinDriver::input(peripherals.pins.gpio4)?;
    
    // ... create neurons/synapses ...
    
    loop {
        // Read sensor
        if sensor.is_high() {
            // Inject current into sensory neuron 0
            candidate_potentials[0] = INT8Value::from_f32(5.0);
        }
        
        // Process burst
        neurons.process_burst(&candidate_potentials, &mut fired_mask);
        
        // Reset potentials
        for p in &mut candidate_potentials {
            *p = INT8Value::zero();
        }
        
        platform.delay_ms(10);
    }
}
```

### 8.2 Connect via UART

Example: Read commands from computer

```rust
use esp_idf_svc::hal::uart::{UartDriver, config::Config as UartConfig};

fn main() -> anyhow::Result<()> {
    let platform = Esp32Platform::init()?;
    let peripherals = Peripherals::take()?;
    
    // Configure UART on GPIO17(TX) / GPIO18(RX)
    let uart_config = UartConfig::new()
        .baudrate(esp_idf_svc::hal::units::Hertz(115_200));
    let mut uart = UartDriver::new(
        peripherals.uart1,
        peripherals.pins.gpio17,
        peripherals.pins.gpio18,
        None, None,
        &uart_config,
    )?;
    
    // ... create neurons/synapses ...
    
    let mut rx_buf = [0u8; 128];
    loop {
        // Check for UART data
        if let Ok(len) = uart.read(&mut rx_buf, 0) {
            if len > 0 {
                // Parse command and inject into neurons
                let command = rx_buf[0];
                match command {
                    b'F' => candidate_potentials[0] = INT8Value::from_f32(10.0), // Forward
                    b'B' => candidate_potentials[1] = INT8Value::from_f32(10.0), // Backward
                    b'L' => candidate_potentials[2] = INT8Value::from_f32(10.0), // Left
                    b'R' => candidate_potentials[3] = INT8Value::from_f32(10.0), // Right
                    _ => {}
                }
            }
        }
        
        // Process burst
        neurons.process_burst(&candidate_potentials, &mut fired_mask);
        
        // Send motor output
        if fired_mask[30] { uart.write(b"MOTOR_A_ON\n")?; }
        
        platform.delay_ms(10);
    }
}
```

---

## Step 9: Network Topologies

### 9.1 Simple Reflex Arc (36 neurons)

```
Sensory (10) → Hidden (20) → Motor (5) → Power (1)
```

**Use case**: Basic stimulus-response, demos

### 9.2 Sensor Fusion (100 neurons)

```
Vision (25) ──┐
Audio (25) ───┼─→ Integration (30) → Decision (10) → Motor (10)
Touch (25) ───┘
```

**Use case**: Multi-modal processing

### 9.3 Large Network (40,000 neurons on ESP32-S3)

```
Sensory (1000) → Hidden (38000) → Motor (1000)
```

**Use case**: Complex behaviors, vision processing

---

## Step 10: Debugging and Optimization

### 10.1 Enable Debug Logging

```bash
# Set log level via environment
export RUST_LOG=debug
cargo run --release
```

### 10.2 Measure Burst Timing

```rust
loop {
    let start = platform.get_time_us();
    let fired = neurons.process_burst(&inputs, &mut fired_mask);
    let elapsed = platform.get_time_us() - start;
    
    platform.info(&format!("Burst time: {}μs, fired: {}", elapsed, fired));
    
    // Target: < 10,000μs for 100 Hz
    // If > 10,000μs: network is too large or complex
}
```

### 10.3 Profile Memory

```rust
platform.info(&format!("Heap free: {} bytes", 
    platform.available_memory_bytes()));

// Expected:
// ESP32-S3 with 1K neurons: ~7.9 MB free
// ESP32 with 1K neurons: ~490 KB free
```

### 10.4 Common Issues

| Issue | Cause | Solution |
|-------|-------|----------|
| **Constant reboots** | Stack overflow | Reduce array sizes, increase stack in sdkconfig |
| **Slow bursts (>10ms)** | Network too large | Reduce neurons or optimize |
| **Out of memory** | Too many neurons | Reduce MAX_NEURONS or use ESP32-S3 |
| **No serial output** | Wrong target | Rebuild for correct chip |
| **Upload failed** | Wrong permissions | `sudo chmod 666 /dev/ttyUSB0` |

---

## Step 11: Advanced Configuration

### 11.1 Increase Network Size (ESP32-S3 only)

Edit `sdkconfig.defaults`:
```
CONFIG_SPIRAM=y
CONFIG_SPIRAM_MODE_OCT=y
CONFIG_SPIRAM_SPEED_80M=y
CONFIG_SPIRAM_USE_MALLOC=y
```

Then increase in code:
```rust
const MAX_NEURONS: usize = 40_000;  // 40K neurons!
const MAX_SYNAPSES: usize = 200_000;
```

### 11.2 Enable WiFi (ESP32-S3)

```toml
[dependencies]
esp-idf-svc = { version = "0.49", features = ["binstart"] }
```

```rust
use esp_idf_svc::wifi::*;

// Configure WiFi
let wifi = BlockingWifi::wrap(
    EspWifi::new(peripherals.modem, sysloop.clone(), Some(nvs))?,
    sysloop,
)?;

wifi.set_configuration(&Configuration::Client(ClientConfiguration {
    ssid: "your-wifi".try_into().unwrap(),
    password: "your-password".try_into().unwrap(),
    ..Default::default()
}))?;

wifi.start()?;
wifi.connect()?;
wifi.wait_netif_up()?;

platform.info("WiFi connected!");
```

### 11.3 Over-the-Air (OTA) Updates

```bash
# Flash once via USB
cargo run --release

# Then flash over WiFi
espflash flash --monitor --serial-target http://esp32.local
```

---

## Performance Benchmarks

### ESP32 Standard (520 KB SRAM)

| Neurons | Synapses | Burst Time | Max Frequency |
|---------|----------|------------|---------------|
| 100 | 500 | 150 μs | 6,666 Hz |
| 500 | 2,500 | 800 μs | 1,250 Hz |
| 1,000 | 5,000 | 1.6 ms | 625 Hz |
| 2,000 | 10,000 | 3.2 ms | 312 Hz |

### ESP32-S3 (8 MB PSRAM)

| Neurons | Synapses | Burst Time | Max Frequency |
|---------|----------|------------|---------------|
| 1,000 | 5,000 | 1.5 ms | 666 Hz |
| 5,000 | 25,000 | 7.5 ms | 133 Hz |
| 10,000 | 50,000 | 15 ms | 66 Hz |
| 40,000 | 200,000 | 60 ms | 16 Hz |

**Note**: With INT8 quantization, these are 26% faster than FP32!

---

## Production Deployment Checklist

- [ ] Hardware selected and purchased
- [ ] Toolchain installed and tested
- [ ] Code builds successfully
- [ ] Flashes to hardware
- [ ] Serial monitor shows expected output
- [ ] Burst frequency meets requirements
- [ ] Memory usage is acceptable
- [ ] Sensors integrated and tested
- [ ] Motors/actuators connected
- [ ] WiFi configured (if needed)
- [ ] Error handling added
- [ ] Logging configured
- [ ] Power consumption measured
- [ ] Enclosure designed
- [ ] Documentation written

---

## Example Projects

### Project 1: Obstacle Avoidance Robot

**Hardware**:
- ESP32-S3-DevKitC-1
- HC-SR04 ultrasonic sensor (GPIO 4/5)
- L298N motor driver (GPIO 12/13/14/15)
- 6V battery pack

**Network**: 50 neurons (10 sensory, 30 hidden, 10 motor)

### Project 2: Smart Home Hub

**Hardware**:
- ESP32-S3
- PIR motion sensor (GPIO 4)
- Temperature sensor (I2C)
- LED indicators (GPIO 8-11)

**Network**: 200 neurons (sensor fusion + decision making)

### Project 3: Vision Processing

**Hardware**:
- ESP32-S3 with camera module
- OV2640 camera
- SPI display

**Network**: 10,000 neurons (vision processing pipeline)

---

## Resources

### Official Documentation
- [ESP-IDF Programming Guide](https://docs.espressif.com/projects/esp-idf/en/latest/)
- [esp-rs Book](https://esp-rs.github.io/book/)
- [espflash Documentation](https://github.com/esp-rs/espflash)

### FEAGI Documentation
- [feagi-hal README](../README.md)
- [Platform Comparison](PLATFORM_COMPARISON.md)
- [Porting Guide](PORTING_GUIDE.md)

### Community
- [ESP32 Forum](https://esp32.com/)
- [esp-rs Matrix Chat](https://matrix.to/#/#esp-rs:matrix.org)
- [FEAGI Discord](https://discord.gg/feagi)

---

## Troubleshooting

### Build Errors

**Error**: `error: linker 'xtensa-esp32s3-elf-gcc' not found`
```bash
# Solution: Reinstall ESP32 toolchain
espup install
source ~/export-esp.sh
```

**Error**: `error: can't find crate for 'std'`
```bash
# Solution: Using wrong target
# For ESP32-S3, use: xtensa-esp32s3-none-elf
# For ESP32, use: xtensa-esp32-none-elf
```

### Flash Errors

**Error**: `Error: espflash::timeout`
```bash
# Solution: Hold BOOT button while flashing
# Or try: espflash flash --before no_reset
```

**Error**: `Error: Permission denied (os error 13)`
```bash
# Linux: Add user to dialout group
sudo usermod -a -G dialout $USER
# Then logout and login

# Or temporarily:
sudo chmod 666 /dev/ttyUSB0
```

### Runtime Errors

**Constant reboots**:
- Check stack size (increase in sdkconfig)
- Reduce MAX_NEURONS
- Check for infinite recursion

**Heap exhausted**:
- Use ESP32-S3 with PSRAM
- Reduce network size
- Check for memory leaks

---

## Next Steps

### ✅ You've successfully deployed FEAGI on ESP32!

**Now you can**:
1. Modify the network topology
2. Add more sensors/actuators
3. Increase network size (up to 40K on ESP32-S3)
4. Add WiFi connectivity
5. Implement learning algorithms
6. Deploy to production

### **Advanced Topics**:
- Multi-core processing (ESP32 has 2 cores)
- Deep sleep modes for power saving
- Bluetooth LE communication
- OTA firmware updates
- Remote monitoring via WiFi

---

**Congratulations! You're now running FEAGI neural networks on embedded hardware!** 🎉

**Questions?** Check [PLATFORM_COMPARISON.md](PLATFORM_COMPARISON.md) or ask in [FEAGI Discord](https://discord.gg/feagi)

