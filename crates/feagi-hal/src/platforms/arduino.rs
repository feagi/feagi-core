// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/// Arduino platform implementations
///
/// Currently supports:
/// - Arduino Due (ARM Cortex-M3, 84 MHz, 96KB SRAM)
///
/// Future support:
/// - Arduino Mega 2560 (AVR, 16 MHz, 8KB SRAM)
/// - Arduino Uno (AVR, 16 MHz, 2KB SRAM)
use crate::hal::*;

// Arduino Due uses generic embedded-hal + cortex-m
// No specific arduino-due-hal crate exists yet
#[cfg(feature = "arduino-due")]
use cortex_m::peripheral::DWT;

/// Arduino Due platform structure
#[cfg(feature = "arduino-due")]
pub struct ArduinoDuePlatform {
    // Note: Arduino HAL uses global peripherals, so we don't store them
    start_time_ms: u32,
}

#[cfg(feature = "arduino-due")]
impl ArduinoDuePlatform {
    /// Initialize Arduino Due platform
    ///
    /// # Returns
    /// Initialized ArduinoDuePlatform
    ///
    /// # Example
    /// ```no_run
    /// let platform = ArduinoDuePlatform::init();
    /// ```
    pub fn init() -> Self {
        // Arduino Due initialization
        // In a real application, this would initialize:
        // - DWT cycle counter for microsecond timing
        // - USART for serial communication
        // - GPIO for pins
        Self { start_time_ms: 0 }
    }

    /// Get Arduino Due chip model
    pub fn chip_model(&self) -> &'static str {
        "Arduino Due (SAM3X8E)"
    }
}

#[cfg(feature = "arduino-due")]
impl TimeProvider for ArduinoDuePlatform {
    fn get_time_us(&self) -> u64 {
        // Arduino Due runs at 84 MHz (ARM Cortex-M3)
        // Use DWT cycle counter for microsecond-precision timing
        // DWT.CYCCNT increments at CPU frequency
        // This is a placeholder - actual implementation would read DWT.CYCCNT
        // and convert to microseconds: cyccnt / (cpu_freq_mhz)
        self.start_time_ms as u64 * 1000
    }

    fn delay_us(&self, us: u32) {
        // Arduino Due: Use busy-wait with nop instructions
        // At 84 MHz: 84 cycles per microsecond
        let cycles = us as u64 * 84;
        for _ in 0..cycles {
            cortex_m::asm::nop();
        }
    }

    fn delay_ms(&self, ms: u32) {
        self.delay_us(ms * 1000);
    }
}

#[cfg(feature = "arduino-due")]
impl SerialIO for ArduinoDuePlatform {
    type Error = ();

    fn write(&mut self, data: &[u8]) -> Result<usize, Self::Error> {
        // Arduino HAL uses global serial
        // This is a simplified implementation
        // Real implementation would need serial peripheral access
        Ok(data.len())
    }

    fn read(&mut self, buffer: &mut [u8]) -> Result<usize, Self::Error> {
        // Non-blocking read would need serial peripheral access
        Ok(0)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[cfg(feature = "arduino-due")]
impl Logger for ArduinoDuePlatform {
    fn log(&self, _level: LogLevel, _message: &str) {
        // Arduino Due: Would write log message to USART
        // In a real implementation, this would:
        // 1. Format the message (using ufmt for no_std)
        // 2. Write to USART peripheral
        // 3. Add timestamp from DWT
        // For now, this is a no-op placeholder
    }
}

#[cfg(feature = "arduino-due")]
impl Platform for ArduinoDuePlatform {
    fn name(&self) -> &'static str {
        "Arduino Due"
    }

    fn cpu_frequency_hz(&self) -> u32 {
        84_000_000 // 84 MHz
    }

    fn available_memory_bytes(&self) -> usize {
        // Arduino Due has 96KB SRAM
        // We can't easily query free memory on Arduino without malloc
        96_000 // Total SRAM
    }
}

// Placeholder for when arduino-due feature is not enabled
#[cfg(not(feature = "arduino-due"))]
pub struct ArduinoDuePlatform;

#[cfg(not(feature = "arduino-due"))]
impl ArduinoDuePlatform {
    pub fn init() -> Result<Self, &'static str> {
        Err("Arduino Due feature not enabled. Rebuild with --features arduino-due")
    }
}

// Note: Arduino Mega and Uno would need avr-hal instead of arduino-hal
// They use AVR architecture, not ARM, so they need different implementation
