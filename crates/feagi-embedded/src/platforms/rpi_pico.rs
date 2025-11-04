/// Raspberry Pi Pico platform implementation
/// 
/// Supports:
/// - Raspberry Pi Pico (RP2040, dual-core ARM Cortex-M0+, 133 MHz, 264KB SRAM)
/// - Raspberry Pi Pico W (RP2040 + wireless)

use crate::hal::*;

/// Raspberry Pi Pico platform structure
#[cfg(feature = "rpi-pico")]
pub struct RpiPicoPlatform {
    // Timer for timing functions
    start_time_us: u64,
}

#[cfg(feature = "rpi-pico")]
impl RpiPicoPlatform {
    /// Initialize Raspberry Pi Pico platform
    /// 
    /// # Returns
    /// Initialized RpiPicoPlatform
    ///
    /// # Example
    /// ```no_run
    /// let platform = RpiPicoPlatform::init().expect("Failed to init");
    /// ```
    pub fn init() -> Result<Self, &'static str> {
        Ok(Self {
            start_time_us: 0,
        })
    }
    
    /// Get Raspberry Pi Pico chip model
    pub fn chip_model(&self) -> &'static str {
        "Raspberry Pi Pico (RP2040)"
    }
}

#[cfg(feature = "rpi-pico")]
impl TimeProvider for RpiPicoPlatform {
    fn get_time_us(&self) -> u64 {
        // RP2040 has a built-in 64-bit microsecond timer
        // This would use rp2040_hal::timer::Timer
        // Placeholder - actual implementation needs timer setup
        self.start_time_us
    }
    
    fn delay_us(&self, us: u32) {
        // RP2040: Use built-in delay functions
        // Assuming 133 MHz default clock
        let cycles = (us as u64 * 133) / 1000;  // cycles = us * (MHz / 1000)
        for _ in 0..cycles {
            cortex_m::asm::nop();
        }
    }
    
    fn delay_ms(&self, ms: u32) {
        self.delay_us(ms * 1000);
    }
}

#[cfg(feature = "rpi-pico")]
impl SerialIO for RpiPicoPlatform {
    type Error = ();
    
    fn write(&mut self, data: &[u8]) -> Result<usize, Self::Error> {
        // RP2040: Would use UART peripheral
        // Placeholder - actual implementation needs UART setup
        Ok(data.len())
    }
    
    fn read(&mut self, buffer: &mut [u8]) -> Result<usize, Self::Error> {
        // Non-blocking read from UART
        Ok(0)
    }
    
    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[cfg(feature = "rpi-pico")]
impl Logger for RpiPicoPlatform {
    fn log(&self, _level: LogLevel, _message: &str) {
        // RP2040: Would write log message via USB CDC or UART
        // In a real implementation, this would:
        // 1. Format the message (using ufmt for no_std)
        // 2. Write to USB CDC (for USB serial) or UART
        // For now, this is a no-op placeholder
    }
}

#[cfg(feature = "rpi-pico")]
impl Platform for RpiPicoPlatform {
    fn name(&self) -> &'static str {
        "Raspberry Pi Pico"
    }
    
    fn cpu_frequency_hz(&self) -> u32 {
        // RP2040 typically runs at 133 MHz
        // Can be overclocked to 250+ MHz
        133_000_000
    }
    
    fn available_memory_bytes(&self) -> usize {
        // RP2040 has 264KB SRAM
        264_000
    }
}

// Placeholder for when rpi-pico feature is not enabled
#[cfg(not(feature = "rpi-pico"))]
pub struct RpiPicoPlatform;

#[cfg(not(feature = "rpi-pico"))]
impl RpiPicoPlatform {
    pub fn init() -> Result<Self, &'static str> {
        Err("Raspberry Pi Pico feature not enabled. Rebuild with --features rpi-pico")
    }
}

